//! The windowed host of subscript-typegpu programs.
//!
//! The host owns the window, the surface, the device, and the event loop (W1). It calls the
//! script's `init`, `frame`, and `shutdown` exports, and it translates window events into plain
//! values (W2, W3). The script encodes one frame per call and never presents.

use std::ffi::c_void;
use std::io::Write;
use std::path::PathBuf;

use facade::surface;
use raw_window_handle::{HasWindowHandle, RawWindowHandle};
use subscript_typegpu_harness::{native as facade, EntryArg, ProgramLoadError, ReloadSession};
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::{ElementState, MouseButton, MouseScrollDelta, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::{Key, NamedKey};
use winit::window::{Window, WindowAttributes, WindowId};

/// The CoreGraphics color-space declarations that the macOS `CAMetalLayer` needs (W9).
#[cfg(target_os = "macos")]
mod macos_colorspace {
    use std::ffi::c_void;

    /// The opaque object that a `CGColorSpaceRef` points to.
    #[repr(C)]
    pub struct CGColorSpace {
        _opaque: [u8; 0],
    }

    // SAFETY: CGColorSpaceRef is encoded as a pointer to the opaque
    // CGColorSpace struct by the Objective-C runtime.
    unsafe impl objc2::RefEncode for CGColorSpace {
        const ENCODING_REF: objc2::Encoding =
            objc2::Encoding::Pointer(&objc2::Encoding::Struct("CGColorSpace", &[]));
    }

    #[link(name = "CoreGraphics", kind = "framework")]
    extern "C" {
        /// The name of the sRGB color space.
        pub static kCGColorSpaceSRGB: *const c_void;
        /// Creates one owned color space from a name, and returns null on failure.
        pub fn CGColorSpaceCreateWithName(name: *const c_void) -> *mut CGColorSpace;
        /// Releases one color space that a create call returned.
        pub fn CGColorSpaceRelease(space: *mut CGColorSpace);
    }
}

/// A failure of one host run (W8).
///
/// `Compile` carries the compiler's diagnostics, which the exit path prints before its one line.
/// `Host` carries a message whose first line names the step that failed.
enum WindowError {
    Compile(ProgramLoadError),
    Host(String),
}

impl From<String> for WindowError {
    /// Treats every host message as a host failure, so that `?` accepts a `Result<_, String>`.
    fn from(message: String) -> Self {
        Self::Host(message)
    }
}

/// Creates the webgpu.h surface for `window` on `instance` (W9).
///
/// macOS attaches a `CAMetalLayer` and sets the layer's color space to sRGB. An unset color space
/// reaches the display in the display's native gamut. Windows passes the `HWND` and the
/// `HINSTANCE`. Another platform returns an error.
///
/// The caller releases the returned surface before it releases the instance (W4).
fn create_surface(
    instance: facade::SubscriptTypegpuInstance,
    window: &Window,
) -> Result<surface::WGPUSurface, String> {
    #[cfg(target_os = "macos")]
    {
        use objc2::ClassType;
        use objc2_app_kit::NSView;
        use objc2_quartz_core::CAMetalLayer;

        let handle = window
            .window_handle()
            .map_err(|error| format!("window handle: {error}"))?;
        let RawWindowHandle::AppKit(handle) = handle.as_raw() else {
            return Err("window handle is not AppKit".to_owned());
        };
        // SAFETY: winit's AppKit handle is a live NSView for the lifetime of
        // the window, and this borrow does not escape the function.
        let view = unsafe { &*handle.ns_view.as_ptr().cast::<NSView>() };
        // SAFETY: +layer is an Objective-C class method returning an
        // autoreleased CAMetalLayer, which NSView retains below.
        let layer: *mut CAMetalLayer = unsafe { objc2::msg_send![CAMetalLayer::class(), layer] };
        if layer.is_null() {
            return Err("CAMetalLayer creation returned null".to_owned());
        }
        // SAFETY: the named constant is live. Create returns one owned color
        // space. setColorspace retains it, and release balances this create once.
        unsafe {
            let color_space =
                macos_colorspace::CGColorSpaceCreateWithName(macos_colorspace::kCGColorSpaceSRGB);
            if color_space.is_null() {
                return Err("sRGB color space creation returned null".to_owned());
            }
            let _: () = objc2::msg_send![layer, setColorspace: color_space];
            macos_colorspace::CGColorSpaceRelease(color_space);
        }
        // SAFETY: `layer` is a live CAMetalLayer and setLayer retains it.
        unsafe { view.setLayer(Some(&*layer)) };
        view.setWantsLayer(true);
        let source = surface::WGPUSurfaceSourceMetalLayer {
            chain: surface::WGPUChainedStruct {
                next: std::ptr::null_mut(),
                sType: surface::WGPUSType_SurfaceSourceMetalLayer,
            },
            layer: layer.cast::<c_void>(),
        };
        let descriptor = surface::WGPUSurfaceDescriptor {
            nextInChain: (&source.chain as *const surface::WGPUChainedStruct).cast_mut(),
            label: surface::WGPUStringView {
                data: std::ptr::null(),
                length: 0,
            },
        };
        let table = surface::table()?;
        // SAFETY: `instance` is live and the descriptor chain remains valid
        // for the duration of the backend call.
        let created = unsafe { (table.wgpuInstanceCreateSurface)(instance, &descriptor) };
        if created.is_null() {
            Err("surface creation returned null".to_owned())
        } else {
            Ok(created)
        }
    }
    #[cfg(target_os = "windows")]
    {
        let handle = window
            .window_handle()
            .map_err(|error| format!("window handle: {error}"))?;
        let RawWindowHandle::Win32(handle) = handle.as_raw() else {
            return Err("window handle is not Win32".to_owned());
        };
        let source = surface::WGPUSurfaceSourceWindowsHWND {
            chain: surface::WGPUChainedStruct {
                next: std::ptr::null_mut(),
                sType: surface::WGPUSType_SurfaceSourceWindowsHWND,
            },
            hinstance: handle
                .hinstance
                .map_or(std::ptr::null_mut(), |value| value.get() as *mut c_void),
            hwnd: handle.hwnd.get() as *mut c_void,
        };
        let descriptor = surface::WGPUSurfaceDescriptor {
            nextInChain: (&source.chain as *const surface::WGPUChainedStruct).cast_mut(),
            label: surface::WGPUStringView {
                data: std::ptr::null(),
                length: 0,
            },
        };
        let table = surface::table()?;
        // SAFETY: `instance` is live and the descriptor chain remains valid
        // for the duration of the backend call.
        let created = unsafe { (table.wgpuInstanceCreateSurface)(instance, &descriptor) };
        if created.is_null() {
            Err("surface creation returned null".to_owned())
        } else {
            Ok(created)
        }
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        let _ = (instance, window);
        Err("surface creation is implemented only for macOS and Windows".to_owned())
    }
}

/// Pumps the instance until the future completes, and returns when it succeeds.
///
/// A failure drops the future, so no slot survives the error. The message names the step and the
/// backend status.
fn await_future(
    instance: facade::SubscriptTypegpuInstance,
    future: facade::SubscriptTypegpuFutureId,
    kind: FutureKind,
) -> Result<(), String> {
    loop {
        let status = facade::subscript_typegpu_future_status(instance, future);
        match status {
            0 => facade::subscript_typegpu_instance_process_events(instance),
            1 => return Ok(()),
            failure => {
                facade::subscript_typegpu_future_drop(instance, future);
                return Err(format!(
                    "{} failed with {} ({failure})",
                    kind.step(),
                    kind.status_name(failure)
                ));
            }
        }
    }
}

/// The two async requests of the host. The value selects the names that a failure message holds.
#[derive(Clone, Copy)]
enum FutureKind {
    RequestAdapter,
    RequestDevice,
}

impl FutureKind {
    /// Returns the step name for the one failure line (W8).
    fn step(self) -> &'static str {
        match self {
            Self::RequestAdapter => "adapter request",
            Self::RequestDevice => "device request",
        }
    }

    /// Returns the pinned header's constant name for one facade future status.
    ///
    /// The facade returns the negated backend status, and -100 for an unknown id (L6). A status
    /// that no arm names reads as `UnknownStatus`.
    fn status_name(self, status: i32) -> &'static str {
        if status == -100 {
            return "UnknownFuture";
        }
        match (self, status.unsigned_abs()) {
            (Self::RequestAdapter, 2) => "WGPURequestAdapterStatus_CallbackCancelled",
            (Self::RequestAdapter, 3) => "WGPURequestAdapterStatus_Unavailable",
            (Self::RequestAdapter, 4) => "WGPURequestAdapterStatus_Error",
            (Self::RequestDevice, 2) => "WGPURequestDeviceStatus_CallbackCancelled",
            (Self::RequestDevice, 3) => "WGPURequestDeviceStatus_Error",
            _ => "UnknownStatus",
        }
    }
}

/// One queued input event that the host delivers before the next frame (W3 Rev 2).
///
/// `Wheel` carries pixel deltas. `KeyDown` and `KeyUp` carry one modifier bit. `Text` carries one
/// Unicode scalar.
enum InputEvent {
    Wheel(f32, f32),
    KeyDown(u32),
    KeyUp(u32),
    Text(u32),
}

/// The state of one windowed run.
///
/// `key`, `pointer_x`, `pointer_y`, and `buttons` are the input that every `frame` call reads
/// (W3). `input_exports` holds the entry names that the script exports, so the host drops an
/// event whose optional entry is absent (W2 Rev 3).
struct Host {
    input_exports: Vec<String>,
    input_events: Vec<InputEvent>,
    session: ReloadSession,
    instance: facade::SubscriptTypegpuInstance,
    surface: surface::WGPUSurface,
    device: facade::SubscriptTypegpuDevice,
    window: Option<Window>,
    format: surface::WGPUTextureFormat,
    width: u32,
    height: u32,
    key: u32,
    pointer_x: f32,
    pointer_y: f32,
    buttons: u32,
    frames: u64,
    frame_limit: Option<u64>,
    initialized: bool,
    exit_requested: bool,
    shutdown_complete: bool,
    result: Result<(), String>,
}

impl Host {
    /// Builds the host state before the event loop starts.
    ///
    /// The pointer starts at `-1, -1`, which tells the script that the pointer never entered the
    /// window (W2 Rev 2). With a `frame_limit` of `None`, the run ends when the window closes.
    fn new(
        session: ReloadSession,
        input_exports: Vec<String>,
        instance: facade::SubscriptTypegpuInstance,
        frame_limit: Option<u64>,
    ) -> Self {
        Self {
            input_exports,
            input_events: Vec::new(),
            session,
            instance,
            surface: std::ptr::null_mut(),
            device: std::ptr::null_mut(),
            window: None,
            format: 0,
            width: 0,
            height: 0,
            key: 0,
            pointer_x: -1.0,
            pointer_y: -1.0,
            buttons: 0,
            frames: 0,
            frame_limit,
            initialized: false,
            exit_requested: false,
            shutdown_complete: false,
            result: Ok(()),
        }
    }

    /// Creates the window, the surface, the adapter, and the device, then configures the surface.
    ///
    /// The function selects the format (W7). It then calls `init` with the instance, the device,
    /// and that format (W2). It drains the async work of `init` and requests the first frame (W6).
    ///
    /// The adapter is released as soon as the capabilities query returns, because the device
    /// alone outlives this function.
    fn initialize(&mut self, event_loop: &ActiveEventLoop) -> Result<(), String> {
        let attributes = WindowAttributes::default()
            .with_title("subscript-typegpu window")
            .with_inner_size(LogicalSize::new(960.0, 640.0));
        let window = event_loop
            .create_window(attributes)
            .map_err(|error| format!("create window: {error}"))?;
        self.window = Some(window);
        self.surface = create_surface(
            self.instance,
            self.window.as_ref().expect("window stored before surface"),
        )?;

        let adapter_future = facade::subscript_typegpu_instance_request_adapter(self.instance);
        if adapter_future == 0 {
            return Err("adapter request returned no future".to_owned());
        }
        await_future(self.instance, adapter_future, FutureKind::RequestAdapter)?;
        let adapter = facade::subscript_typegpu_request_adapter_take(self.instance, adapter_future);
        if adapter.is_null() {
            return Err("adapter request returned null".to_owned());
        }

        let device_future = facade::subscript_typegpu_adapter_request_device_with_descriptor(
            self.instance,
            adapter,
            std::ptr::null(),
        );
        if device_future == 0 {
            facade::subscript_typegpu_adapter_release(adapter);
            return Err("device request returned no future".to_owned());
        }
        if let Err(error) = await_future(self.instance, device_future, FutureKind::RequestDevice) {
            facade::subscript_typegpu_adapter_release(adapter);
            return Err(error);
        }
        self.device = facade::subscript_typegpu_request_device_take(self.instance, device_future);
        if self.device.is_null() {
            facade::subscript_typegpu_adapter_release(adapter);
            return Err("device request returned null".to_owned());
        }

        let table = surface::table()?;
        // SAFETY: the all-zero capabilities value is the webgpu.h initializer
        // and the backend fills it for the live surface and adapter.
        let mut capabilities: surface::WGPUSurfaceCapabilities = unsafe { std::mem::zeroed() };
        // SAFETY: all handles are live and `capabilities` is writable.
        let status =
            unsafe { (table.wgpuSurfaceGetCapabilities)(self.surface, adapter, &mut capabilities) };
        facade::subscript_typegpu_adapter_release(adapter);
        if status != surface::WGPUStatus_Success {
            return Err(format!("surface capabilities failed with status {status}"));
        }
        // W7: bgra8unorm when the surface lists it, and the first listed format otherwise.
        self.format = if capabilities.formatCount == 0 || capabilities.formats.is_null() {
            // 0 is WGPUTextureFormat_Undefined, which the check below rejects.
            0
        } else {
            // SAFETY: a successful capabilities query supplies formatCount
            // readable values until FreeMembers is called below.
            let formats = unsafe {
                std::slice::from_raw_parts(capabilities.formats, capabilities.formatCount)
            };
            if formats.contains(&surface::WGPUTextureFormat_BGRA8Unorm) {
                surface::WGPUTextureFormat_BGRA8Unorm
            } else {
                formats[0]
            }
        };
        // SAFETY: the capabilities value came from the matching backend and is
        // freed exactly once after its arrays have been inspected.
        unsafe { (table.wgpuSurfaceCapabilitiesFreeMembers)(capabilities) };
        if self.format == 0 {
            return Err("surface reports no texture format".to_owned());
        }
        self.configure()?;
        // The flag precedes the call, so a script that fails inside `init` still receives its
        // `shutdown` call (W2).
        self.initialized = true;
        self.call_entry(
            "init",
            &[
                EntryArg::Handle(self.instance.cast::<c_void>()),
                EntryArg::Handle(self.device.cast::<c_void>()),
                EntryArg::I32(self.format as i32),
            ],
        )?;
        self.drain_async()?;
        self.window
            .as_ref()
            .expect("window stored")
            .request_redraw();
        Ok(())
    }

    /// Configures the surface for the current window size, with Fifo presentation.
    ///
    /// A window can report a zero extent, so the width and the height hold a minimum of 1.
    fn configure(&mut self) -> Result<(), String> {
        let size = self
            .window
            .as_ref()
            .ok_or_else(|| "configure without a window".to_owned())?
            .inner_size();
        self.width = size.width.max(1);
        self.height = size.height.max(1);
        let config = surface::WGPUSurfaceConfiguration {
            nextInChain: std::ptr::null_mut(),
            device: self.device,
            format: self.format,
            usage: surface::WGPUTextureUsage_RenderAttachment,
            width: self.width,
            height: self.height,
            viewFormatCount: 0,
            viewFormats: std::ptr::null(),
            alphaMode: surface::WGPUCompositeAlphaMode_Auto,
            presentMode: surface::WGPUPresentMode_Fifo,
        };
        let table = surface::table()?;
        // SAFETY: the surface, device, and configuration are live for this call.
        unsafe { (table.wgpuSurfaceConfigure)(self.surface, &config) };
        Ok(())
    }

    /// Writes the script's buffered `print` output to the host's stdout and flushes it (W6).
    fn write_output(&mut self) -> Result<(), String> {
        let output = self.session.take_output();
        let mut stdout = std::io::stdout().lock();
        stdout
            .write_all(&output)
            .and_then(|()| stdout.flush())
            .map_err(|error| format!("write script output: {error}"))
    }

    /// Calls one script entry and writes the output that the call produced.
    ///
    /// The output reaches stdout even when the call fails, so the script's own line stays visible
    /// before the host's error line (W6).
    fn call_entry(&mut self, name: &str, args: &[EntryArg]) -> Result<(), String> {
        let called = self
            .session
            .call_export_with(name, args)
            .map_err(|error| format!("{name}: {error}"));
        let output = self.write_output();
        called?;
        output
    }

    /// Pumps the facade and steps the session until the script holds no pending async work (W6).
    ///
    /// The pump runs before the first step, so a request that the entry started can complete
    /// without another frame. The output reaches stdout even when a step fails.
    fn drain_async(&mut self) -> Result<(), String> {
        let drained: Result<(), String> = (|| {
            facade::subscript_typegpu_instance_process_events(self.instance);
            while self.session.async_pending() != 0 {
                self.session
                    .async_step()
                    .map_err(|error| format!("async: {error}"))?;
                facade::subscript_typegpu_instance_process_events(self.instance);
            }
            Ok(())
        })();
        let output = self.write_output();
        drained?;
        output
    }

    /// Delivers the queued input events in event order, then leaves the queue empty (W3 Rev 2).
    ///
    /// An event whose optional entry the script does not export is dropped.
    fn deliver_input(&mut self) -> Result<(), String> {
        for event in std::mem::take(&mut self.input_events) {
            let (name, args) = match event {
                InputEvent::Wheel(x, y) => ("wheel", vec![EntryArg::F32(x), EntryArg::F32(y)]),
                InputEvent::KeyDown(key) => ("keyDown", vec![EntryArg::U32(key)]),
                InputEvent::KeyUp(key) => ("keyUp", vec![EntryArg::U32(key)]),
                InputEvent::Text(point) => ("textInput", vec![EntryArg::U32(point)]),
            };
            if self.input_exports.iter().any(|entry| entry == name) {
                self.call_entry(name, &args)?;
            }
        }
        Ok(())
    }

    /// Acquires one surface texture, calls `frame`, and presents the result (W5).
    ///
    /// `Timeout` skips the frame and `Outdated` reconfigures first. Both keep the input queue, so
    /// it reaches the next presented frame (W3 Rev 3). `Lost` and `Error` end the run.
    ///
    /// The view and the texture are released on every path. The key slot clears after a
    /// presented frame (W3).
    fn frame(&mut self) -> Result<(), String> {
        let table = surface::table()?;
        // SAFETY: zero is the webgpu.h initializer for the out structure.
        let mut acquired: surface::WGPUSurfaceTexture = unsafe { std::mem::zeroed() };
        // SAFETY: the surface is configured and `acquired` is writable.
        unsafe { (table.wgpuSurfaceGetCurrentTexture)(self.surface, &mut acquired) };
        match acquired.status {
            surface::WGPUSurfaceGetCurrentTextureStatus_Timeout => {
                self.drain_async()?;
                return Ok(());
            }
            surface::WGPUSurfaceGetCurrentTextureStatus_Outdated => {
                self.configure()?;
                self.drain_async()?;
                return Ok(());
            }
            surface::WGPUSurfaceGetCurrentTextureStatus_Lost => {
                return Err("surface lost".to_owned());
            }
            surface::WGPUSurfaceGetCurrentTextureStatus_Error => {
                return Err("surface acquisition error".to_owned());
            }
            surface::WGPUSurfaceGetCurrentTextureStatus_SuccessOptimal
            | surface::WGPUSurfaceGetCurrentTextureStatus_SuccessSuboptimal => {}
            status => return Err(format!("surface acquisition status {status}")),
        }
        // W5: the host never calls `frame` without a texture.
        if acquired.texture.is_null() {
            return Err("surface acquisition returned null texture".to_owned());
        }
        let view =
            facade::subscript_typegpu_texture_create_view(acquired.texture, std::ptr::null());
        if view.is_null() {
            facade::subscript_typegpu_texture_release(acquired.texture);
            return Err("surface texture view creation returned null".to_owned());
        }
        let called = self.deliver_input().and_then(|()| {
            self.call_entry(
                "frame",
                &[
                    EntryArg::Handle(view.cast::<c_void>()),
                    EntryArg::U32(self.width),
                    EntryArg::U32(self.height),
                    EntryArg::U32(self.key),
                    EntryArg::F32(self.pointer_x),
                    EntryArg::F32(self.pointer_y),
                    EntryArg::U32(self.buttons),
                ],
            )
        });
        if let Err(error) = called {
            facade::subscript_typegpu_texture_view_release(view);
            facade::subscript_typegpu_texture_release(acquired.texture);
            return Err(error);
        }
        // SAFETY: the surface has one acquired texture ready for presentation.
        let presented = unsafe { (table.wgpuSurfacePresent)(self.surface) };
        facade::subscript_typegpu_texture_view_release(view);
        facade::subscript_typegpu_texture_release(acquired.texture);
        if presented != surface::WGPUStatus_Success {
            return Err(format!("surface present failed with status {presented}"));
        }
        self.frames += 1;
        // The presented frame read the key slot, so the next frame starts with no key (W3).
        self.key = 0;
        self.drain_async()?;
        Ok(())
    }

    fn reached_frame_limit(&self) -> bool {
        self.frame_limit.is_some_and(|limit| self.frames >= limit)
    }

    /// Records the run's result and asks the event loop to exit.
    ///
    /// The first call wins, so a later event cannot replace the recorded failure. The shutdown
    /// sequence runs on the loop's exiting path (W8 Rev 2).
    fn finish(&mut self, event_loop: &ActiveEventLoop, result: Result<(), String>) {
        if self.exit_requested {
            return;
        }
        self.result = result;
        self.exit_requested = true;
        event_loop.exit();
    }

    /// Calls `shutdown`, then releases the device, the surface, and the instance in that order
    /// (W4).
    ///
    /// The body runs once. It keeps the first error, so a release failure never hides the failure
    /// that ended the run. The frame count prints only after a run with no error (W11).
    fn shutdown(&mut self) {
        if self.shutdown_complete {
            return;
        }
        self.shutdown_complete = true;
        let mut result = std::mem::replace(&mut self.result, Ok(()));
        if self.initialized {
            if let Err(error) = self.call_entry("shutdown", &[]) {
                if result.is_ok() {
                    result = Err(error);
                }
            }
        }
        if !self.device.is_null() {
            facade::subscript_typegpu_device_release(self.device);
            self.device = std::ptr::null_mut();
        }
        if !self.surface.is_null() {
            match surface::table() {
                Ok(table) => {
                    // SAFETY: this host owns the live surface and releases it once.
                    unsafe { (table.wgpuSurfaceRelease)(self.surface) };
                }
                Err(error) if result.is_ok() => result = Err(error),
                Err(_) => {}
            }
            self.surface = std::ptr::null_mut();
        }
        if !self.instance.is_null() {
            facade::subscript_typegpu_instance_release(self.instance);
            self.instance = std::ptr::null_mut();
        }
        // W4: the window goes last, because the surface holds a layer of the window.
        self.window.take();
        if result.is_ok() {
            println!("window:frames={}", self.frames);
        }
        self.result = result;
    }

    /// Records `error` as the run's result and asks the event loop to exit.
    fn fail(&mut self, event_loop: &ActiveEventLoop, error: String) {
        self.finish(event_loop, Err(error));
    }
}

impl ApplicationHandler for Host {
    /// Runs the one-time initialization on the first resume of the application.
    ///
    /// A limit of zero frames ends the run here, so `--frames 0` covers the whole path with no
    /// presented frame.
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() && !self.exit_requested {
            match self.initialize(event_loop) {
                Ok(()) if self.reached_frame_limit() => self.finish(event_loop, Ok(())),
                Ok(()) => {}
                Err(error) => self.fail(event_loop, error),
            }
        }
    }

    /// Translates one window event into host state, into a queued input event, or into one frame.
    ///
    /// A close ends the loop and a resize reconfigures the surface (W3). A redraw request encodes
    /// and presents one frame (W5). The pointer position and the button bits are level state, so
    /// nothing clears them.
    ///
    /// The host ignores an event of another window and every event after the exit request.
    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        if self.window.as_ref().map(Window::id) != Some(window_id) || self.exit_requested {
            return;
        }
        match event {
            WindowEvent::CloseRequested => self.finish(event_loop, Ok(())),
            WindowEvent::Resized(size) if size.width != 0 && size.height != 0 => {
                if let Err(error) = self.configure() {
                    self.fail(event_loop, error);
                }
            }
            WindowEvent::KeyboardInput { event, .. } => {
                let pressed = event.state == ElementState::Pressed;
                if pressed {
                    // W3: one slot holds the key. A second press before the next frame replaces
                    // the first.
                    self.key = match &event.logical_key {
                        Key::Character(value) => value.chars().next().map_or(0, u32::from),
                        Key::Named(NamedKey::Space) => u32::from(' '),
                        _ => 0,
                    };
                }
                // The modifier bit set of W3 Rev 2. A key repeat counts as a press.
                let bit = match &event.logical_key {
                    Key::Named(NamedKey::Shift) => Some(1),
                    Key::Named(NamedKey::Control) => Some(2),
                    Key::Named(NamedKey::Alt) => Some(4),
                    Key::Named(NamedKey::Backspace) => Some(8),
                    Key::Named(NamedKey::Enter) => Some(16),
                    _ => None,
                };
                if let Some(bit) = bit {
                    self.input_events.push(if pressed {
                        InputEvent::KeyDown(bit)
                    } else {
                        InputEvent::KeyUp(bit)
                    });
                }
                if pressed {
                    // W3 Rev 3: the text comes from the produced text of the event, so a control
                    // chord produces none. A control character is not text either.
                    if let Some(text) = &event.text {
                        self.input_events.extend(
                            text.chars()
                                .filter(|point| !point.is_control())
                                .map(|point| InputEvent::Text(u32::from(point))),
                        );
                    }
                }
            }
            WindowEvent::MouseWheel { delta, .. } => {
                // W3 Rev 2: one line is 30 pixels, and a pixel delta passes as is.
                let (x, y) = match delta {
                    MouseScrollDelta::LineDelta(x, y) => (x * 30.0, y * 30.0),
                    MouseScrollDelta::PixelDelta(position) => {
                        (position.x as f32, position.y as f32)
                    }
                };
                self.input_events.push(InputEvent::Wheel(x, y));
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.pointer_x = position.x as f32;
                self.pointer_y = position.y as f32;
            }
            // The pointer position is level state (W3), so a leave event keeps the last position.
            WindowEvent::CursorLeft { .. } => {}
            WindowEvent::MouseInput { state, button, .. } => {
                // The button bit set of W2 Rev 2: bit 0 left, bit 1 right, bit 2 middle.
                let bit = match button {
                    MouseButton::Left => Some(1_u32 << 0),
                    MouseButton::Right => Some(1_u32 << 1),
                    MouseButton::Middle => Some(1_u32 << 2),
                    _ => None,
                };
                if let Some(bit) = bit {
                    match state {
                        ElementState::Pressed => self.buttons |= bit,
                        ElementState::Released => self.buttons &= !bit,
                    }
                }
            }
            WindowEvent::RedrawRequested => match self.frame() {
                Ok(()) if self.reached_frame_limit() => self.finish(event_loop, Ok(())),
                Ok(()) => {
                    // Each frame asks for the next one, so the host runs a continuous loop.
                    if let Some(window) = &self.window {
                        window.request_redraw();
                    }
                }
                Err(error) => self.fail(event_loop, error),
            },
            _ => {}
        }
    }

    /// Runs the shutdown sequence on the event loop's exiting path.
    ///
    /// An application quit takes this path too, so a quit reports like a close (W8 Rev 2).
    fn exiting(&mut self, _event_loop: &ActiveEventLoop) {
        self.shutdown();
    }
}

/// Reads the program path and the optional frame limit from the command line (W11).
///
/// The path defaults to the example program. A repeated `--frames`, an unknown argument, or a
/// missing count returns the usage line.
fn arguments() -> Result<(PathBuf, Option<u64>), String> {
    let mut arguments = std::env::args_os().skip(1);
    let program = arguments
        .next()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("examples/window-triangle/main.ts"));
    let mut frame_limit = None;
    while let Some(argument) = arguments.next() {
        if argument != "--frames" || frame_limit.is_some() {
            return Err("usage: subscript-typegpu-window <program.ts> [--frames <n>]".to_owned());
        }
        let value = arguments
            .next()
            .ok_or_else(|| "--frames requires a count".to_owned())?;
        frame_limit = Some(
            value
                .to_string_lossy()
                .parse::<u64>()
                .map_err(|_| "--frames requires a non-negative integer".to_owned())?,
        );
    }
    Ok((program, frame_limit))
}

/// Builds the event loop on Windows, where the host runs on a thread with the compiler's stack.
///
/// The host starts the loop off the main thread there, so the builder must accept any thread.
#[cfg(windows)]
fn event_loop() -> Result<EventLoop<()>, winit::error::EventLoopError> {
    use winit::platform::windows::EventLoopBuilderExtWindows;

    EventLoop::builder().with_any_thread(true).build()
}

/// Builds the event loop on the main thread (W10).
#[cfg(not(windows))]
fn event_loop() -> Result<EventLoop<()>, winit::error::EventLoopError> {
    EventLoop::new()
}

/// Compiles the program, creates the instance, and runs the event loop to its end.
///
/// A compile failure carries its diagnostics, which the exit path prints before the one host line
/// (W8). The returned error is the result that the host recorded.
fn run() -> Result<(), WindowError> {
    let (program, frame_limit) = arguments()?;
    let (session, exports) = subscript_typegpu_harness::load_program_with_exports(&program)
        .map_err(WindowError::Compile)?;
    let instance = facade::subscript_typegpu_create_instance();
    if instance.is_null() {
        return Err("instance creation returned null".to_owned().into());
    }
    let event_loop = event_loop().map_err(|error| format!("event loop: {error}"))?;
    let mut host = Host::new(session, exports, instance, frame_limit);
    event_loop
        .run_app(&mut host)
        .map_err(|error| format!("event loop: {error}"))?;
    host.result?;
    Ok(())
}

/// Runs the host, prints one line for a failure, and exits with a non-zero code (W8).
///
/// Windows runs the host on a thread with the compiler's stack. Every other platform runs it on
/// the main thread (W10).
fn main() {
    #[cfg(windows)]
    let result = subscript_typegpu_harness::run_on_compiler_stack(run)
        .map_err(WindowError::Host)
        .and_then(|result| result);
    #[cfg(not(windows))]
    let result = run();
    match result {
        Ok(()) => {}
        Err(WindowError::Compile(error)) => {
            if let Some(diagnostics) = error.diagnostics() {
                eprintln!("{diagnostics}");
            }
            eprintln!("window:{}", error.summary());
            std::process::exit(1);
        }
        Err(WindowError::Host(error)) => {
            let first_line = error.lines().next().unwrap_or("unknown failure");
            eprintln!("window:{first_line}");
            std::process::exit(1);
        }
    }
}
