use imgui::Ui;
use crate::db::ProjectDB;
use crate::transformer::CodeTransformer;
use crate::learn::UserLearner;

pub fn draw_ui(
    ui: &Ui,
    db: &ProjectDB,
    transformer: &mut CodeTransformer,
    learner: &mut UserLearner,
) {
    ui.window("VERSO K1 - AI Coding Assistant")
        .size([420.0, 640.0], imgui::Condition::FirstUseEver)
        .position([10.0, 10.0], imgui::Condition::FirstUseEver)
        .build(|| {
            ui.text("AI Powered Code Assistant");
            ui.separator();

            // === Projects ===
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

            // === Learning ===
            ui.text("Learning Patterns:");
            for pattern in learner.get_top_patterns(5) {
                ui.text(format!("- {}: {}x", pattern.action, pattern.count));
            }

            if ui.button("Record: Opened File") {
                learner.record_action("open_file", "coding_session");
            }

            ui.separator();

            // === Transformer Test ===
            if ui.button("Test Transformer") {
                let tokens = vec![1u32, 2, 3, 4, 5];
                let out = transformer.understand_code(&tokens);
                ui.text(format!("Shape: {:?}", out.shape()));
            }

            ui.separator();

            // === Suggestions ===
            ui.text("Suggestion:");
            if let Some(suggestion) = learner.suggest_next("coding_session") {
                ui.text(format!("Next: {}", suggestion));
            } else {
                ui.text_disabled("Not enough data...");
            }
        });
}
