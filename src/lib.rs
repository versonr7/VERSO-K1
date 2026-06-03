use glow::HasContext as _;
use android_activity::{AndroidApp, MainEvent, PollEvent, InputStatus};
use android_activity::input::{InputEvent, MotionAction};
use khronos_egl as egl;
use log::{info, error};
use std::time::Duration;

mod db;
mod transformer;
mod learn;
mod ui;

#[link(name = "EGL")]
extern "C" {}

#[no_mangle]
extern "C" fn android_main(app: AndroidApp) {
    android_logger::init_once(
        android_logger::Config::default().with_max_level(log::LevelFilter::Info)
    );
    println!("VERSO K1 BOOT");
    info!("=== VERSO K1 Booting ===");
    run_app(app);
}

fn run_app(app: AndroidApp) {
    let db = match db::ProjectDB::new() {
        Ok(d) => { info!("DB initialized"); d }
        Err(e) => { error!("DB init failed: {}", e); return; }
    };

    let mut transformer = transformer::CodeTransformer::new(128, 64, 4);
    let mut learner = learn::UserLearner::new();

    let egl = egl::Instance::new(egl::Static);

    let mut egl_display: Option<egl::Display> = None;
    let mut egl_surface: Option<egl::Surface> = None;
    let mut egl_context: Option<egl::Context> = None;
    let mut gl: Option<glow::Context> = None;
    let mut imgui_ctx: Option<imgui::Context> = None;
    let mut renderer: Option<imgui_glow_renderer::Renderer> = None;
    let mut texture_map: Option<imgui_glow_renderer::SimpleTextureMap> = None;

    let mut touch_x: f32 = 0.0;
    let mut touch_y: f32 = 0.0;
    let mut touch_down: bool = false;

    let mut keyboard_buffer: String = String::with_capacity(4096);
    let mut transformer_result: Option<String> = None;

    let mut running = true;

    while running {
        app.poll_events(Some(Duration::from_millis(0)), |event| {
            match event {
                PollEvent::Main(main_event) => {
                    match main_event {
                        MainEvent::InitWindow { .. } => {
                            if egl_display.is_some() { return; }
                            info!("=== InitWindow: initializing EGL ===");

                            let display = unsafe {
                                egl.get_display(egl::DEFAULT_DISPLAY)
                                    .expect("egl.get_display failed")
                            };

                            match egl.initialize(display) {
                                Ok((major, minor)) => info!("EGL initialized: {}.{}", major, minor),
                                Err(e) => { error!("egl.initialize failed: {:?}", e); return; }
                            }

                            let mut all_configs: Vec<egl::Config> = Vec::with_capacity(64);
                            if let Err(e) = egl.get_configs(display, &mut all_configs) {
                                error!("egl.get_configs failed: {:?}", e); return;
                            }
                            info!("EGL configs returned: {}", all_configs.len());

                            if all_configs.is_empty() {
                                error!("No EGL configs returned by system");
                                return;
                            }

                            let config = match all_configs.iter().find(|&&c| {
                                match egl.get_config_attrib(display, c, egl::SURFACE_TYPE) {
                                    Ok(st) => (st & egl::WINDOW_BIT) != 0,
                                    Err(_) => false,
                                }
                            }) {
                                Some(&c) => { info!("Found config with WINDOW_BIT"); c }
                                None => {
                                    info!("No WINDOW_BIT config, using first available");
                                    match all_configs.into_iter().next() {
                                        Some(c) => c,
                                        None => { error!("Could not get first config"); return; }
                                    }
                                }
                            };

                            let native_window = match app.native_window() {
                                Some(nw) => nw,
                                None => { error!("No native window in InitWindow"); return; }
                            };

                            let surface = unsafe {
                                match egl.create_window_surface(
                                    display, config,
                                    native_window.ptr().as_ptr() as egl::NativeWindowType,
                                    None,
                                ) {
                                    Ok(s) => s,
                                    Err(e) => { error!("create_window_surface failed: {:?}", e); return; }
                                }
                            };

                            let ctx_attribs = [egl::CONTEXT_CLIENT_VERSION, 2, egl::NONE];
                            let context = match egl.create_context(display, config, None, &ctx_attribs) {
                                Ok(c) => c,
                                Err(e) => { error!("create_context failed: {:?}", e); return; }
                            };

                            if let Err(e) = egl.make_current(display, Some(surface), Some(surface), Some(context)) {
                                error!("make_current failed: {:?}", e); return;
                            }

                            let gl_ctx = unsafe {
                                glow::Context::from_loader_function(|s| {
                                    egl.get_proc_address(s)
                                        .map(|p| p as *const _)
                                        .unwrap_or(std::ptr::null())
                                })
                            };

                            let mut imgui = imgui::Context::create();
                            {
                                let mut io = imgui.io_mut();
                                io.display_size = [native_window.width() as f32, native_window.height() as f32];
                                io.display_framebuffer_scale = [1.0, 1.0];
                            }
                            let mut tex_map = imgui_glow_renderer::SimpleTextureMap::default();
                            let rend = match imgui_glow_renderer::Renderer::initialize(&gl_ctx, &mut imgui, &mut tex_map, true) {
                                Ok(r) => r,
                                Err(e) => { error!("Renderer init failed: {:?}", e); return; }
                            };

                            egl_display = Some(display);
                            egl_surface = Some(surface);
                            egl_context = Some(context);
                            gl = Some(gl_ctx);
                            imgui_ctx = Some(imgui);
                            renderer = Some(rend);
                            texture_map = Some(tex_map);
                            info!("EGL vendor: {:?}", egl.query_string(Some(display), egl::VENDOR));
                            info!("EGL version: {:?}", egl.query_string(Some(display), egl::VERSION));
                            info!("=== EGL/GL/ImGui ready ===");
                        }
                        MainEvent::TerminateWindow { .. } => {
                            info!("=== TerminateWindow ===");
                            if let Some(display) = egl_display.take() {
                                if let Some(context) = egl_context.take() {
                                    let _ = egl.destroy_context(display, context);
                                }
                                if let Some(surface) = egl_surface.take() {
                                    let _ = egl.destroy_surface(display, surface);
                                }
                                let _ = egl.terminate(display);
                            }
                            gl = None; imgui_ctx = None; renderer = None; texture_map = None;
                        }
                        MainEvent::Destroy => { running = false; }
                        _ => {}
                    }
                }
                _ => {}
            }
        });

        if !running { break; }

        // === TOUCH INPUT ===
        if let Ok(mut iter) = app.input_events_iter() {
            loop {
                let read_input = iter.next(|event| {
                    if let InputEvent::MotionEvent(motion) = event {
                        let pointer = motion.pointer_at_index(0);
                        touch_x = pointer.x();
                        touch_y = pointer.y();
                        match motion.action() {
                            MotionAction::Down | MotionAction::Move => touch_down = true,
                            MotionAction::Up | MotionAction::Cancel => touch_down = false,
                            _ => {}
                        }
                    }
                    InputStatus::Unhandled
                });
                if !read_input { break; }
            }
        }

        // === RENDER ===
        if let (Some(display), Some(surface), Some(gl_ctx), Some(imgui), Some(rend), Some(tex_map)) =
            (egl_display, egl_surface, gl.as_ref(), imgui_ctx.as_mut(), renderer.as_mut(), texture_map.as_mut())
        {
            unsafe {
                gl_ctx.clear_color(1.0, 0.0, 0.0, 1.0);
                gl_ctx.clear(glow::COLOR_BUFFER_BIT);
            }
            let io = imgui.io_mut();
            io.mouse_pos = [touch_x, touch_y];
            io.mouse_down[0] = touch_down;
            if let Some(nw) = app.native_window() {
                let w = nw.width() as f32;
                let h = nw.height() as f32;
                if io.display_size[0] != w || io.display_size[1] != h {
                    io.display_size = [w, h];
                }
            }
            let display_size = io.display_size;
            ui::draw_ui(&imgui.frame(), &db, &mut transformer, &mut learner, &mut keyboard_buffer, display_size, &mut transformer_result);

            let draw_data = imgui.render();
            if let Err(e) = rend.render(gl_ctx, tex_map, draw_data) {
                error!("Render error: {:?}", e);
            }
            if let Err(e) = egl.swap_buffers(display, surface) {
                error!("swap_buffers failed: {:?}", e);
            }
        }

        std::thread::sleep(std::time::Duration::from_millis(16));
    }
}

#[cfg(test)]
mod android_tests {
    use std::panic;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, Ordering};

    #[test]
    fn test_panic_hook_works() {
        let panic_called = Arc::new(AtomicBool::new(false));
        let panic_called_clone = Arc::clone(&panic_called);
        let old_hook = panic::take_hook();
        panic::set_hook(Box::new(move |_| {
            panic_called_clone.store(true, Ordering::SeqCst);
        }));
        let _ = panic::catch_unwind(|| { panic!("test"); });
        panic::set_hook(old_hook);
        assert!(panic_called.load(Ordering::SeqCst), "Panic hook should be called");
    }
}
