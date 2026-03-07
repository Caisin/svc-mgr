//! Environment variable management operations for TUI

use super::app::{App, Mode};

impl App {
    pub fn enter_add_env_mode(&mut self) {
        self.mode = Mode::AddEnv;
        self.edit_key.clear();
        self.edit_value.clear();
        self.status_message = "Add environment variable - Key: (type and press Tab to continue)".to_string();
    }

    pub fn switch_to_value_input(&mut self) {
        self.status_message = format!("Add environment variable - Value for {}: (Enter to save, Esc to cancel)", self.edit_key);
    }

    pub fn save_new_env(&mut self) {
        if self.edit_key.is_empty() {
            self.status_message = "Error: Key cannot be empty".to_string();
            self.mode = Mode::Normal;
            return;
        }

        let manager = crate::env::manager();
        match manager.set(crate::env::EnvScope::User, &self.edit_key, &self.edit_value) {
            Ok(_) => {
                self.status_message = format!("Added {}={}", self.edit_key, self.edit_value);
                self.refresh_env_vars();
            }
            Err(e) => {
                self.status_message = format!("Error adding env var: {}", e);
            }
        }
        self.mode = Mode::Normal;
        self.edit_key.clear();
        self.edit_value.clear();
    }

    pub fn delete_env_var(&mut self) {
        if let Some(idx) = self.env_list_state.selected() {
            let env_vars = self.filtered_env_vars();
            if let Some((key, _)) = env_vars.get(idx) {
                let manager = crate::env::manager();
                match manager.unset(crate::env::EnvScope::User, key) {
                    Ok(_) => {
                        self.status_message = format!("Deleted environment variable: {}", key);
                        self.refresh_env_vars();
                    }
                    Err(e) => {
                        self.status_message = format!("Error deleting env var: {}", e);
                    }
                }
            }
        }
    }

    pub fn save_edit(&mut self) {
        let manager = crate::env::manager();
        match manager.set(crate::env::EnvScope::User, &self.edit_key, &self.edit_value) {
            Ok(_) => {
                self.status_message = format!("Updated {}={}", self.edit_key, self.edit_value);
                self.refresh_env_vars();
            }
            Err(e) => {
                self.status_message = format!("Error updating env var: {}", e);
            }
        }
        self.mode = Mode::Normal;
        self.edit_value.clear();
        self.edit_key.clear();
    }
}
