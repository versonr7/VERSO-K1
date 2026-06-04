use macroquad::prelude::*;
use log::info;

mod db;
mod transformer;
mod learn;

#[no_mangle]
extern "C" fn android_main() {
    android_logger::init_once(
        android_logger::Config::default().with_max_level(log::LevelFilter::Info)
    );
    info!("VERSO K1 Booting...");
    main();
}

#[macroquad::main("VERSO K1")]
async fn main() {
    let db = match db::ProjectDB::new() {
        Ok(d) => { info!("DB initialized"); d }
        Err(e) => { info!("DB init failed: {}", e); return; }
    };

    let mut transformer = transformer::CodeTransformer::new(128, 64, 4);
    let mut learner = learn::UserLearner::new();
    let mut keyboard_buffer = String::new();
    let mut transformer_result: Option<String> = None;

    loop {
        clear_background(Color::from_rgba(8, 8, 18, 255));

        // === Top Bar ===
        draw_rectangle(0.0, 0.0, screen_width(), 40.0, Color::from_rgba(20, 20, 40, 200));
        let _ = draw_text("VERSO K1 - AI Assistant", 20.0, 28.0, 24.0, WHITE);

        // === Chat Area ===
        let chat_y = 60.0;
        draw_rectangle(10.0, chat_y, screen_width() - 20.0, 200.0, Color::from_rgba(30, 30, 50, 150));
        let _ = draw_text("Chat:", 20.0, chat_y + 25.0, 20.0, WHITE);

        let mut y_offset = chat_y + 50.0;
        if !keyboard_buffer.is_empty() {
            let _ = draw_text(&format!("You: {}", keyboard_buffer), 20.0, y_offset, 18.0, GREEN);
            y_offset += 25.0;
        }
        if let Some(ref result) = transformer_result {
            let _ = draw_text(&format!("AI: {}", result), 20.0, y_offset, 18.0, YELLOW);
        }

        // === Input Field ===
        let input_y = chat_y + 220.0;
        draw_rectangle(10.0, input_y, screen_width() - 20.0, 50.0, Color::from_rgba(40, 40, 60, 200));
        let _ = draw_text(&format!("> {}", keyboard_buffer), 20.0, input_y + 35.0, 20.0, WHITE);

        // === Buttons ===
        let btn_y = input_y + 70.0;
        if draw_button("Send", 10.0, btn_y, 100.0, 40.0, BLUE) && !keyboard_buffer.is_empty() {
            learner.record_action("send_message", "chat");
            transformer_result = Some(format!("Processed: {}", keyboard_buffer));
            keyboard_buffer.clear();
        }

        if draw_button("Clear", 120.0, btn_y, 100.0, 40.0, RED) {
            keyboard_buffer.clear();
            transformer_result = None;
        }

        if draw_button("Test AI", 230.0, btn_y, 100.0, 40.0, PURPLE) {
            let tokens = vec![1u32, 2, 3, 4, 5];
            let out = transformer.understand_code(&tokens);
            transformer_result = Some(format!("Shape: {:?}", out.shape()));
        }

        // === Projects ===
        let proj_y = btn_y + 60.0;
        let _ = draw_text("Projects:", 20.0, proj_y, 20.0, WHITE);
        match db.get_projects() {
            Ok(projects) => {
                let mut p_y = proj_y + 30.0;
                for p in projects {
                    let _ = draw_text(&format!("- {} ({})", p.name, p.language), 30.0, p_y, 16.0, GRAY);
                    p_y += 22.0;
                }
            }
            Err(e) => {
                let _ = draw_text(&format!("DB Error: {}", e), 30.0, proj_y + 30.0, 16.0, RED);
            }
        }

        if draw_button("Remember", 10.0, proj_y + 120.0, 120.0, 35.0, DARKGREEN) {
            let _ = db.remember_project("current", "/data/current", "rust");
        }

        // === Learning ===
        let learn_y = proj_y + 170.0;
        let _ = draw_text("Learning:", 20.0, learn_y, 20.0, WHITE);
        let mut l_y = learn_y + 30.0;
        for pattern in learner.get_top_patterns(5) {
            let _ = draw_text(&format!("- {}: {}x", pattern.action, pattern.count), 30.0, l_y, 16.0, GRAY);
            l_y += 22.0;
        }

        if draw_button("Record", 10.0, learn_y + 130.0, 120.0, 35.0, ORANGE) {
            learner.record_action("open_file", "coding_session");
        }

        // === Keyboard Input ===
        if let Some(c) = get_char_pressed() {
            keyboard_buffer.push(c);
        }
        if is_key_pressed(KeyCode::Backspace) && !keyboard_buffer.is_empty() {
            keyboard_buffer.pop();
        }
        if is_key_pressed(KeyCode::Enter) && !keyboard_buffer.is_empty() {
            learner.record_action("send_message", "chat");
            transformer_result = Some(format!("Sent: {}", keyboard_buffer));
            keyboard_buffer.clear();
        }
        if is_key_pressed(KeyCode::Space) {
            keyboard_buffer.push(' ');
        }

        next_frame().await;
    }
}

fn draw_button(label: &str, x: f32, y: f32, w: f32, h: f32, color: Color) -> bool {
    let mouse = mouse_position();
    let hovered = mouse.0 >= x && mouse.0 <= x + w && mouse.1 >= y && mouse.1 <= y + h;
    
    let btn_color = if hovered {
        Color::new(color.r * 1.2, color.g * 1.2, color.b * 1.2, color.a)
    } else {
        color
    };
    
    draw_rectangle(x, y, w, h, btn_color);
    draw_rectangle_lines(x, y, w, h, 2.0, WHITE);
    
    let text_dim = measure_text(label, None, 20, 1.0);
    let _ = draw_text(label, x + (w - text_dim.width) / 2.0, y + h / 2.0 + 7.0, 20.0, WHITE);
    
    hovered && is_mouse_button_pressed(MouseButton::Left)
}

#[cfg(test)]
mod tests {
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
