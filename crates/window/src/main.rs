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

#[cfg(target_os = "macos")]
mod macos_colorspace {
    use std::ffi::c_void;

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
        pub static kCGColorSpaceSRGB: *const c_void;
        pub fn CGColorSpaceCreateWithName(name: *const c_void) -> *mut CGColorSpace;
        pub fn CGColorSpaceRelease(space: *mut CGColorSpace);
    }
}

enum WindowError {
    Compile(ProgramLoadError),
    Host(String),
}

impl From<String> for WindowError {
    fn from(message: String) -> Self {
        Self::Host(message)
    }
}

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

#[derive(Clone, Copy)]
enum FutureKind {
    RequestAdapter,
    RequestDevice,
}

impl FutureKind {
    fn step(self) -> &'static str {
        match self {
            Self::RequestAdapter => "adapter request",
            Self::RequestDevice => "device request",
        }
    }

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

enum InputEvent {
    Wheel(f32, f32),
    KeyDown(u32),
    KeyUp(u32),
    Text(u32),
}

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
        self.format = if capabilities.formatCount == 0 || capabilities.formats.is_null() {
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

    fn write_output(&mut self) -> Result<(), String> {
        let output = self.session.take_output();
        let mut stdout = std::io::stdout().lock();
        stdout
            .write_all(&output)
            .and_then(|()| stdout.flush())
            .map_err(|error| format!("write script output: {error}"))
    }

    fn call_entry(&mut self, name: &str, args: &[EntryArg]) -> Result<(), String> {
        let called = self
            .session
            .call_export_with(name, args)
            .map_err(|error| format!("{name}: {error}"));
        let output = self.write_output();
        called?;
        output
    }

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
        self.key = 0;
        self.drain_async()?;
        Ok(())
    }

    fn reached_frame_limit(&self) -> bool {
        self.frame_limit.is_some_and(|limit| self.frames >= limit)
    }

    fn finish(&mut self, event_loop: &ActiveEventLoop, result: Result<(), String>) {
        if self.exit_requested {
            return;
        }
        self.result = result;
        self.exit_requested = true;
        event_loop.exit();
    }

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
        self.window.take();
        if result.is_ok() {
            println!("window:frames={}", self.frames);
        }
        self.result = result;
    }

    fn fail(&mut self, event_loop: &ActiveEventLoop, error: String) {
        self.finish(event_loop, Err(error));
    }
}

impl ApplicationHandler for Host {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() && !self.exit_requested {
            match self.initialize(event_loop) {
                Ok(()) if self.reached_frame_limit() => self.finish(event_loop, Ok(())),
                Ok(()) => {}
                Err(error) => self.fail(event_loop, error),
            }
        }
    }

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
                    self.key = match &event.logical_key {
                        Key::Character(value) => value.chars().next().map_or(0, u32::from),
                        Key::Named(NamedKey::Space) => u32::from(' '),
                        _ => 0,
                    };
                }
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
            WindowEvent::CursorLeft { .. } => {}
            WindowEvent::MouseInput { state, button, .. } => {
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
                    if let Some(window) = &self.window {
                        window.request_redraw();
                    }
                }
                Err(error) => self.fail(event_loop, error),
            },
            _ => {}
        }
    }

    fn exiting(&mut self, _event_loop: &ActiveEventLoop) {
        self.shutdown();
    }
}

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

#[cfg(windows)]
fn event_loop() -> Result<EventLoop<()>, winit::error::EventLoopError> {
    use winit::platform::windows::EventLoopBuilderExtWindows;

    EventLoop::builder().with_any_thread(true).build()
}

#[cfg(not(windows))]
fn event_loop() -> Result<EventLoop<()>, winit::error::EventLoopError> {
    EventLoop::new()
}

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

fn main() {
    #[cfg(windows)]
    let result = subscript_typegpu_harness::run_on_compiler_stack(run);
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
