// src/expert/bevy_expert.rs ~=#######D]======A===r===c====M===o===o===n=====<Lord[EXPERT]Xyn>=====S===t===u===d===i===o===s======[R|$>
// src/expert/bevy_expert.rs

use crate::constants::CRATE_VERSIONS;
use crate::omnixtracker::OmniXError;
use anyhow::{Context, Result};
use bevy::prelude::*;
use log::{error, info, warn};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BevyExpert {
    knowledge_base: Vec<String>,
    best_practices: Vec<String>,
    libraries: HashMap<String, String>,
}

impl BevyExpert {
    pub fn new() -> Self {
        let mut expert = Self {
            knowledge_base: Vec::new(),
            best_practices: Vec::new(),
            libraries: HashMap::new(),
        };
        expert.initialize();
        expert
    }

    fn initialize(&mut self) {
        self.load_knowledge_base();
        self.load_best_practices();
        self.load_libraries();
        info!("BevyExpert initialized with knowledge base and libraries.");
    }

    fn load_knowledge_base(&mut self) {
        self.knowledge_base = vec![
            "Bevy entity-component-system architecture".to_string(),
            "Rendering with Bevy's PBR materials".to_string(),
            "Handling input events in Bevy".to_string(),
            "Bevy scheduling and system organization".to_string(),
        ];
        info!("Knowledge base loaded with {} entries", self.knowledge_base.len());
    }

    fn load_best_practices(&mut self) {
        self.best_practices = vec![
            "Keep systems modular and reusable".to_string(),
            "Use components to encapsulate behavior".to_string(),
            "Leverage Bevy's asset system for loading resources".to_string(),
            "Organize code by separating logic into plugins".to_string(),
        ];
        info!("Best practices loaded with {} entries", self.best_practices.len());
    }

    fn load_libraries(&mut self) {
        let crate_versions = CRATE_VERSIONS
            .try_read()
            .expect("Failed to read CRATE_VERSIONS");
        self.libraries.insert(
            "bevy".to_string(),
            crate_versions
                .get("bevy")
                .cloned()
                .unwrap_or_else(|| "0.8".to_string()),
        );
        self.libraries.insert(
            "bevy_mod_picking".to_string(),
            crate_versions
                .get("bevy_mod_picking")
                .cloned()
                .unwrap_or_else(|| "0.3".to_string()),
        );
        self.libraries.insert(
            "bevy_inspector_egui".to_string(),
            crate_versions
                .get("bevy_inspector_egui")
                .cloned()
                .unwrap_or_else(|| "0.7".to_string()),
        );
        info!("Bevy libraries loaded: {:?}", self.libraries);
    }

    pub fn generate_plugin(&self, plugin_name: &str) -> Result<String, OmniXError> {
        let corrected_plugin_name = self.refactor_arguments(plugin_name)?;
        let struct_name = self.to_pascal_case(&corrected_plugin_name);

        let plugin_code = format!(
            r#"
use bevy::prelude::*;

#[derive(Default)]
pub struct {0};

impl Plugin for {0} {{
    fn build(&self, app: &mut App) {{
        app.add_startup_system(setup)
           .add_system(update);
    }}
}}

fn setup(mut commands: Commands) {{
    // Add your setup logic here
}}

fn update() {{
    // Add your update logic here
}}
"#,
            struct_name
        );

        Ok(plugin_code)
    }

    fn refactor_arguments(&self, input: &str) -> Result<String, OmniXError> {
        let cleaned_input: String = input
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == '_')
            .collect();

        if cleaned_input.is_empty() {
            return Err(OmniXError::InvalidInput("Input cannot be empty after correction".into()));
        }

        Ok(self.to_snake_case(&cleaned_input))
    }

    fn to_pascal_case(&self, s: &str) -> String {
        s.split('_')
            .map(|word| {
                let mut c = word.chars();
                match c.next() {
                    None => String::new(),
                    Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
                }
            })
            .collect()
    }

    fn to_snake_case(&self, s: &str) -> String {
        let mut result = String::new();
        let mut prev_is_uppercase = false;
        for (i, c) in s.char_indices() {
            if c.is_uppercase() {
                if i > 0 && !prev_is_uppercase {
                    result.push('_');
                }
                result.push(c.to_ascii_lowercase());
                prev_is_uppercase = true;
            } else {
                result.push(c);
                prev_is_uppercase = false;
            }
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_plugin() {
        let expert = BevyExpert::new();
        let plugin_code = expert.generate_plugin("test_plugin").unwrap();
        assert!(plugin_code.contains("pub struct TestPlugin"));
        assert!(plugin_code.contains("impl Plugin for TestPlugin"));
    }

    #[test]
    fn test_refactor_arguments() {
        let expert = BevyExpert::new();
        assert_eq!(expert.refactor_arguments("testPlugin").unwrap(), "test_plugin");
        assert_eq!(expert.refactor_arguments("TEST_PLUGIN").unwrap(), "test_plugin");
    }
}        