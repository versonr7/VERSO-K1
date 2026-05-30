use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct UserPattern {
    pub action: String,
    pub context: String,
    pub count: u32,
}

pub struct UserLearner {
    patterns: HashMap<String, UserPattern>,
    history: Vec<String>,
}

impl UserLearner {
    pub fn new() -> Self {
        Self {
            patterns: HashMap::new(),
            history: Vec::new(),
        }
    }

    pub fn record_action(&mut self, action: &str, context: &str) {
        let key = format!("{}:{}", action, context);
        let entry = self.patterns.entry(key.clone()).or_insert(UserPattern {
            action: action.to_string(),
            context: context.to_string(),
            count: 0,
        });
        entry.count += 1;
        self.history.push(key);
    }

    pub fn get_top_patterns(&self, n: usize) -> Vec<&UserPattern> {
        let mut patterns: Vec<&UserPattern> = self.patterns.values().collect();
        patterns.sort_by(|a, b| b.count.cmp(&a.count));
        patterns.into_iter().take(n).collect()
    }

    pub fn suggest_next(&self, current_context: &str) -> Option<String> {
        let mut best = None;
        let mut best_count = 0;
        for pattern in self.patterns.values() {
            if pattern.context == current_context && pattern.count > best_count {
                best_count = pattern.count;
                best = Some(pattern.action.clone());
            }
        }
        best
    }

    pub fn save_to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(&self.patterns)
    }

    pub fn load_from_json(&mut self, json: &str) -> Result<(), serde_json::Error> {
        let loaded: HashMap<String, UserPattern> = serde_json::from_str(json)?;
        self.patterns = loaded;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_and_suggest() {
        let mut learner = UserLearner::new();
        learner.record_action("open_file", "rust_project");
        learner.record_action("open_file", "rust_project");
        learner.record_action("run_build", "rust_project");
        let top = learner.get_top_patterns(2);
        assert_eq!(top[0].action, "open_file");
        assert_eq!(top[0].count, 2);
    }

    #[test]
    fn test_suggest_next() {
        let mut learner = UserLearner::new();
        learner.record_action("edit_cargo_toml", "new_project");
        learner.record_action("edit_cargo_toml", "new_project");
        let suggestion = learner.suggest_next("new_project");
        assert_eq!(suggestion, Some("edit_cargo_toml".to_string()));
    }

    #[test]
    fn test_json_roundtrip() {
        let mut learner = UserLearner::new();
        learner.record_action("test", "context");
        let json = learner.save_to_json().unwrap();
        let mut learner2 = UserLearner::new();
        learner2.load_from_json(&json).unwrap();
        assert_eq!(learner2.get_top_patterns(1)[0].count, 1);
    }
}
