use std::ffi::c_void;
use std::path::{Path, PathBuf};

use raw_window_handle::{HasWindowHandle, RawWindowHandle};
use subscript_codegen::{EntryArg, ReloadSession};
use subscript_compiler::SourceFile;
use subscript_typegpu_facade as facade;
use subscript_typegpu_facade::surface;
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::{ElementState, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::{Key, NamedKey};
use winit::window::{Window, WindowAttributes, WindowId};

fn program_files(program: &Path) -> Result<Vec<SourceFile>, String> {
    let mut files = subscript_typegpu_harness::program_files(program)?;
    let generated = subscript_typegpu_gen::generate(&files)
        .map_err(|diagnostics| subscript_compiler::render_diagnostics(&files, &diagnostics))?;
    files.push(SourceFile::new("main.typegpu.ts", generated.support_module));
    Ok(files)
}

fn load_session(program: &Path) -> Result<ReloadSession, String> {
    let files = program_files(program)?;
    ReloadSession::new_with_native_libraries(&files, &[subscript_typegpu_harness::facade_library()])
        .map_err(|error| format!("compile: {error}"))
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
        view.setWantsLayer(true);
        // SAFETY: `layer` is a live CAMetalLayer and setLayer retains it.
        unsafe { view.setLayer(Some(&*layer)) };
        let source = surface::WGPUSurfaceSourceMetalLayer {
            chain: surface::WGPUChainedStruct {
                next: std::ptr::null(),
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
                next: std::ptr::null(),
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
    step: &str,
) -> Result<(), String> {
    loop {
        let status = facade::subscript_typegpu_future_status(instance, future);
        if status == 1 {
            return Ok(());
        }
        if status < 0 {
            facade::subscript_typegpu_future_drop(instance, future);
            return Err(format!("{step} failed with status {status}"));
        }
        facade::subscript_typegpu_instance_process_events(instance);
    }
}

struct Host {
    session: ReloadSession,
    instance: facade::SubscriptTypegpuInstance,
    surface: surface::WGPUSurface,
    device: facade::SubscriptTypegpuDevice,
    window: Option<Window>,
    format: surface::WGPUTextureFormat,
    width: u32,
    height: u32,
    key: u32,
    frames: u64,
    initialized: bool,
    finished: bool,
    result: Result<(), String>,
}

impl Host {
    fn new(session: ReloadSession, instance: facade::SubscriptTypegpuInstance) -> Self {
        Self {
            session,
            instance,
            surface: std::ptr::null_mut(),
            device: std::ptr::null_mut(),
            window: None,
            format: 0,
            width: 0,
            height: 0,
            key: 0,
            frames: 0,
            initialized: false,
            finished: false,
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
        let size = window.inner_size();
        self.width = size.width.max(1);
        self.height = size.height.max(1);
        self.surface = create_surface(self.instance, &window)?;

        let adapter_future = facade::subscript_typegpu_instance_request_adapter(self.instance);
        if adapter_future == 0 {
            return Err("adapter request returned no future".to_owned());
        }
        await_future(self.instance, adapter_future, "adapter request")?;
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
        if let Err(error) = await_future(self.instance, device_future, "device request") {
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
        self.window = Some(window);
        self.initialized = true;
        self.session
            .call_export_with(
                "init",
                &[
                    EntryArg::Handle(self.instance.cast::<c_void>()),
                    EntryArg::Handle(self.device.cast::<c_void>()),
                    EntryArg::I32(self.format as i32),
                ],
            )
            .map_err(|error| format!("init: {error}"))?;
        self.drain_async()?;
        self.window
            .as_ref()
            .expect("window stored")
            .request_redraw();
        Ok(())
    }

    fn configure(&self) -> Result<(), String> {
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

    fn drain_async(&mut self) -> Result<(), String> {
        facade::subscript_typegpu_instance_process_events(self.instance);
        while self.session.async_pending() != 0 {
            self.session
                .async_step()
                .map_err(|error| format!("async: {error}"))?;
            facade::subscript_typegpu_instance_process_events(self.instance);
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
        let called = self.session.call_export_with(
            "frame",
            &[
                EntryArg::Handle(view.cast::<c_void>()),
                EntryArg::U32(self.width),
                EntryArg::U32(self.height),
                EntryArg::U32(self.key),
            ],
        );
        if let Err(error) = called {
            facade::subscript_typegpu_texture_view_release(view);
            facade::subscript_typegpu_texture_release(acquired.texture);
            return Err(format!("frame: {error}"));
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
        if acquired.status == surface::WGPUSurfaceGetCurrentTextureStatus_SuccessSuboptimal {
            self.configure()?;
        }
        Ok(())
    }

    fn finish(&mut self, event_loop: &ActiveEventLoop, result: Result<(), String>) {
        if self.finished {
            return;
        }
        let mut result = result;
        if self.initialized {
            if let Err(error) = self.session.call_export_with("shutdown", &[]) {
                if result.is_ok() {
                    result = Err(format!("shutdown: {error}"));
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
        self.result = result;
        self.finished = true;
        event_loop.exit();
    }

    fn fail(&mut self, event_loop: &ActiveEventLoop, error: String) {
        self.finish(event_loop, Err(error));
    }
}

impl ApplicationHandler for Host {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() && !self.finished {
            if let Err(error) = self.initialize(event_loop) {
                self.fail(event_loop, error);
            }
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        if self.window.as_ref().map(Window::id) != Some(window_id) || self.finished {
            return;
        }
        match event {
            WindowEvent::CloseRequested => self.finish(event_loop, Ok(())),
            WindowEvent::Resized(size) if size.width != 0 && size.height != 0 => {
                self.width = size.width;
                self.height = size.height;
                if let Err(error) = self.configure() {
                    self.fail(event_loop, error);
                }
            }
            WindowEvent::KeyboardInput { event, .. } if event.state == ElementState::Pressed => {
                self.key = match event.logical_key {
                    Key::Character(value) => value.chars().next().map_or(0, u32::from),
                    Key::Named(NamedKey::Space) => u32::from(' '),
                    _ => 0,
                };
            }
            WindowEvent::RedrawRequested => match self.frame() {
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
}

fn run() -> Result<u64, String> {
    let program = std::env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("examples/window-triangle/main.ts"));
    let session = load_session(&program)?;
    let instance = facade::subscript_typegpu_create_instance();
    if instance.is_null() {
        return Err("instance creation returned null".to_owned());
    }
    let event_loop = EventLoop::new().map_err(|error| format!("event loop: {error}"))?;
    let mut host = Host::new(session, instance);
    event_loop
        .run_app(&mut host)
        .map_err(|error| format!("event loop: {error}"))?;
    host.result?;
    Ok(host.frames)
}

fn main() {
    match run() {
        Ok(frames) => println!("window:frames={frames}"),
        Err(error) => {
            let first_line = error.lines().next().unwrap_or("unknown failure");
            eprintln!("window:{first_line}");
            std::process::exit(1);
        }
    }
}
