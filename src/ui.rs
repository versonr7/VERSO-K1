use crate::db::ProjectDB;
use crate::learn::UserLearner;
use crate::transformer::CodeTransformer;
use imgui::Ui;
use log::info;

pub fn draw_ui(
    ui: &Ui,
    db: &ProjectDB,
    transformer: &mut CodeTransformer,
    learner: &mut UserLearner,
    keyboard_buffer: &mut String,
    display_size: [f32; 2],
) {
    info!("=== draw_ui called ===");
    ui.window("VERSO K1 - AI Coding Assistant")
        .size([display_size[0] - 20.0, display_size[1] - 20.0], imgui::Condition::Always)
        .position([10.0, 10.0], imgui::Condition::FirstUseEver)
        .build(|| {
            ui.text("AI Powered Code Assistant");
            ui.separator();

            ui.text("Chat:");
            ui.separator();

            ui.text("Type (physical keyboard):");
            let mut display_buf = if keyboard_buffer.is_empty() {
                "...".to_string()
            } else {
                keyboard_buffer.clone()
            };
            ui.input_text_multiline("##chat_input", &mut display_buf, [400.0, 80.0])
                .read_only(true)
                .build();

            if ui.button("Send") && !keyboard_buffer.is_empty() {
                learner.record_action("send_message", "chat");
                keyboard_buffer.clear();
            }
            ui.same_line();
            if ui.button("Clear") {
                keyboard_buffer.clear();
            }

            ui.separator();

            if ui.collapsing_header("Projects", imgui::TreeNodeFlags::empty()) {
                match db.get_projects() {
                    Ok(projects) => {
                        if projects.is_empty() {
                            ui.text_disabled("No projects yet...");
                        }
                        for p in projects {
                            ui.text(format!("- {} ({})", p.name, p.language));
                        }
                    }
                    Err(e) => ui.text(format!("DB Error: {}", e)),
                }
            }

            if ui.button("Remember Project") {
                let _ = db.remember_project("current", "/data/current", "rust");
            }

            ui.separator();

            ui.text("Learning Patterns:");
            for pattern in learner.get_top_patterns(5) {
                ui.text(format!("- {}: {}x", pattern.action, pattern.count));
            }

            if ui.button("Record: Opened File") {
                learner.record_action("open_file", "coding_session");
            }

            ui.separator();

            if ui.button("Test Transformer") {
                let tokens = vec![1u32, 2, 3, 4, 5];
                let out = transformer.understand_code(&tokens);
                ui.text(format!("Shape: {:?}", out.shape()));
            }

            ui.separator();

            ui.text("Suggestion:");
            if let Some(suggestion) = learner.suggest_next("coding_session") {
                ui.text(format!("Next: {}", suggestion));
            } else {
                ui.text_disabled("Not enough data...");
            }
        });
}
