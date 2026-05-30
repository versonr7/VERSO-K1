use android_activity::AndroidApp;
use android_activity::input::{InputEvent, MotionAction};
use imgui::Context;
use imgui_glow_renderer::Renderer;
use khronos_egl as egl;
use log::info;

mod db;
mod transformer;
mod learn;
mod ui;

#[link(name = "EGL")]
extern "C" {}

#[no_mangle]
fn android_main(app: AndroidApp) {
    android_logger::init_once(
        android_logger::Config::default().with_max_level(log::LevelFilter::Debug)
    );
    info!("VERSO K1 Booting...");

    let db = db::ProjectDB::new().expect("DB init failed");
    let mut transformer = transformer::CodeTransformer::new(128, 64, 4);
    let mut learner = learn::UserLearner::new();

    let egl = egl::Instance::new(egl::Static);
    let display = unsafe { egl.get_display(egl::DEFAULT_DISPLAY).unwrap() };
    egl.initialize(display).unwrap();

    let attribs = [
        egl::RENDERABLE_TYPE, egl::OPENGL_ES2_BIT,
        egl::SURFACE_TYPE, egl::WINDOW_BIT,
        egl::BLUE_SIZE, 8,
        egl::GREEN_SIZE, 8,
        egl::RED_SIZE, 8,
        egl::NONE,
    ];
    let mut configs = Vec::new();
    egl.choose_config(display, &attribs, &mut configs).unwrap();
    let config = configs.into_iter().next().unwrap();

    let native_window = app.native_window().expect("No native window");
    let surface = unsafe {
        egl.create_window_surface(
            display,
            config,
            native_window.ptr().as_ptr() as egl::NativeWindowType,
            None,
        ).unwrap()
    };

    let ctx_attribs = [egl::CONTEXT_CLIENT_VERSION, 2, egl::NONE];
    let context = egl.create_context(display, config, None, &ctx_attribs).unwrap();
    egl.make_current(display, Some(surface), Some(surface), Some(context)).unwrap();

    let gl = unsafe {
        glow::Context::from_loader_function(|s| {
            egl.get_proc_address(s).map(|p| p as *const _)
                .unwrap_or(std::ptr::null())
        })
    };

    let mut imgui = Context::create();
    let mut texture_map = imgui_glow_renderer::SimpleTextureMap::default();
    let mut renderer = Renderer::initialize(&gl, &mut imgui, &mut texture_map, true)
        .expect("Renderer failed");

    // Touch state
    let mut touch_x: f32 = 0.0;
    let mut touch_y: f32 = 0.0;
    let mut touch_down: bool = false;

    loop {
        // 1. Process input
        if let Ok(mut iter) = app.input_events_iter() {
            while let Some(event) = iter.next() {
                match event {
                    InputEvent::MotionEvent(motion) => {
                        touch_x = motion.x() as f32;
                        touch_y = motion.y() as f32;
                        touch_down = match motion.action() {
                            MotionAction::Down | MotionAction::Move => true,
                            MotionAction::Up | MotionAction::Cancel => false,
                            _ => touch_down,
                        };
                    }
                    _ => {}
                }
            }
        }

        // 2. Update imgui IO with touch
        {
            let io = imgui.io_mut();
            io.mouse_pos = [touch_x, touch_y];
            io.mouse_down[0] = touch_down;
        }

        // 3. Build UI
        let ui = imgui.frame();
        ui::draw_ui(&ui, &db, &mut transformer, &mut learner);

        // 4. Render
        let draw_data = imgui.render();
        renderer.render(&gl, &mut texture_map, draw_data).expect("Render error");

        egl.swap_buffers(display, surface).unwrap();
    }
}
