use std::io;

use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    event::{DisableMouseCapture, EnableMouseCapture},
};

use ratatui::widgets::ListState;

use crate::{ServiceManager, TypedServiceManager};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Services,
    Environment,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Normal,
    Search,
    Edit,
}

pub struct App {
    pub tab: Tab,
    pub mode: Mode,
    pub services: Vec<String>,
    pub env_vars: Vec<(String, String)>,
    pub service_list_state: ListState,
    pub env_list_state: ListState,
    pub search_query: String,
    pub edit_value: String,
    pub status_message: String,
    pub should_quit: bool,
}

impl App {
    pub fn new() -> Self {
        let mut app = Self {
            tab: Tab::Services,
            mode: Mode::Normal,
            services: Vec::new(),
            env_vars: Vec::new(),
            service_list_state: ListState::default(),
            env_list_state: ListState::default(),
            search_query: String::new(),
            edit_value: String::new(),
            status_message: "Press '/' to search, 'e' to edit, 'q' to quit".to_string(),
            should_quit: false,
        };
        app.refresh_services();
        app.refresh_env_vars();
        app
    }

    pub fn refresh_services(&mut self) {
        match TypedServiceManager::native() {
            Ok(manager) => match manager.list() {
                Ok(action) => match action.exec() {
                    Ok(output) => match output.into_list() {
                        Ok(services) => {
                            self.services = services;
                            self.status_message = format!("Loaded {} services", self.services.len());
                        }
                        Err(e) => {
                            self.status_message = format!("Error parsing services: {}", e);
                        }
                    },
                    Err(e) => {
                        self.status_message = format!("Error executing list: {}", e);
                    }
                },
                Err(e) => {
                    self.status_message = format!("Error creating list action: {}", e);
                }
            },
            Err(e) => {
                self.status_message = format!("Error initializing manager: {}", e);
            }
        }
    }

    pub fn refresh_env_vars(&mut self) {
        let manager = crate::env::manager();
        match manager.list(crate::env::EnvScope::User) {
            Ok(vars) => {
                self.env_vars = vars.into_iter().collect();
                self.env_vars.sort_by(|a, b| a.0.cmp(&b.0));
                self.status_message = format!("Loaded {} environment variables", self.env_vars.len());
            }
            Err(e) => {
                self.status_message = format!("Error loading env vars: {}", e);
            }
        }
    }

    pub fn filtered_services(&self) -> Vec<String> {
        if self.search_query.is_empty() {
            self.services.clone()
        } else {
            self.services
                .iter()
                .filter(|s| s.to_lowercase().contains(&self.search_query.to_lowercase()))
                .cloned()
                .collect()
        }
    }

    pub fn filtered_env_vars(&self) -> Vec<(String, String)> {
        if self.search_query.is_empty() {
            self.env_vars.clone()
        } else {
            self.env_vars
                .iter()
                .filter(|(k, v)| {
                    k.to_lowercase().contains(&self.search_query.to_lowercase())
                        || v.to_lowercase().contains(&self.search_query.to_lowercase())
                })
                .cloned()
                .collect()
        }
    }

    pub fn next_item(&mut self) {
        match self.tab {
            Tab::Services => {
                let items = self.filtered_services();
                if items.is_empty() {
                    return;
                }
                let i = match self.service_list_state.selected() {
                    Some(i) => {
                        if i >= items.len() - 1 {
                            0
                        } else {
                            i + 1
                        }
                    }
                    None => 0,
                };
                self.service_list_state.select(Some(i));
            }
            Tab::Environment => {
                let items = self.filtered_env_vars();
                if items.is_empty() {
                    return;
                }
                let i = match self.env_list_state.selected() {
                    Some(i) => {
                        if i >= items.len() - 1 {
                            0
                        } else {
                            i + 1
                        }
                    }
                    None => 0,
                };
                self.env_list_state.select(Some(i));
            }
        }
    }

    pub fn previous_item(&mut self) {
        match self.tab {
            Tab::Services => {
                let items = self.filtered_services();
                if items.is_empty() {
                    return;
                }
                let i = match self.service_list_state.selected() {
                    Some(i) => {
                        if i == 0 {
                            items.len() - 1
                        } else {
                            i - 1
                        }
                    }
                    None => 0,
                };
                self.service_list_state.select(Some(i));
            }
            Tab::Environment => {
                let items = self.filtered_env_vars();
                if items.is_empty() {
                    return;
                }
                let i = match self.env_list_state.selected() {
                    Some(i) => {
                        if i == 0 {
                            items.len() - 1
                        } else {
                            i - 1
                        }
                    }
                    None => 0,
                };
                self.env_list_state.select(Some(i));
            }
        }
    }

    pub fn switch_tab(&mut self) {
        self.tab = match self.tab {
            Tab::Services => Tab::Environment,
            Tab::Environment => Tab::Services,
        };
    }

    pub fn enter_search_mode(&mut self) {
        self.mode = Mode::Search;
        self.search_query.clear();
        self.status_message = "Search mode: Type to filter, Enter to confirm, Esc to cancel".to_string();
    }

    pub fn exit_search_mode(&mut self) {
        self.mode = Mode::Normal;
        self.status_message = "Press '/' to search, 'e' to edit, 'q' to quit".to_string();
    }

    pub fn enter_edit_mode(&mut self) {
        match self.tab {
            Tab::Services => {
                if let Some(idx) = self.service_list_state.selected() {
                    let services = self.filtered_services();
                    if let Some(service) = services.get(idx) {
                        self.status_message = format!("Opening editor for service: {}", service);
                        if let Err(e) = self.edit_service(service) {
                            self.status_message = format!("Error editing service: {}", e);
                        }
                    }
                }
            }
            Tab::Environment => {
                if let Some(idx) = self.env_list_state.selected() {
                    let env_vars = self.filtered_env_vars();
                    if let Some((key, value)) = env_vars.get(idx) {
                        self.mode = Mode::Edit;
                        self.edit_value = value.clone();
                        self.status_message = format!("Editing {}: Type new value, Enter to save, Esc to cancel", key);
                    }
                }
            }
        }
    }

    pub fn save_edit(&mut self) {
        if let Some(idx) = self.env_list_state.selected() {
            let env_vars = self.filtered_env_vars();
            if let Some((key, _)) = env_vars.get(idx) {
                let manager = crate::env::manager();
                match manager.set(crate::env::EnvScope::User, key, &self.edit_value) {
                    Ok(_) => {
                        self.status_message = format!("Updated {}={}", key, self.edit_value);
                        self.refresh_env_vars();
                    }
                    Err(e) => {
                        self.status_message = format!("Error updating env var: {}", e);
                    }
                }
            }
        }
        self.mode = Mode::Normal;
        self.edit_value.clear();
    }

    pub fn cancel_edit(&mut self) {
        self.mode = Mode::Normal;
        self.edit_value.clear();
        self.status_message = "Edit cancelled".to_string();
    }

    fn edit_service(&self, label: &str) -> Result<(), Box<dyn std::error::Error>> {
        // Temporarily exit TUI to run editor
        disable_raw_mode()?;
        execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture)?;

        let label_parsed = label.parse::<crate::ServiceLabel>()?;
        let manager = TypedServiceManager::native()?;
        let action = manager.info(&label_parsed)?;
        let output = action.exec()?;
        let info = output.into_info()?;

        if info.config_path.is_empty() {
            println!("Error: This backend does not use configuration files");
            println!("Press Enter to continue...");
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
        } else {
            let editor = std::env::var("EDITOR")
                .or_else(|_| std::env::var("VISUAL"))
                .unwrap_or_else(|_| {
                    #[cfg(target_os = "windows")]
                    {
                        "notepad".to_string()
                    }
                    #[cfg(not(target_os = "windows"))]
                    {
                        "vi".to_string()
                    }
                });

            std::process::Command::new(&editor)
                .arg(&info.config_path)
                .status()?;
        }

        // Re-enter TUI
        enable_raw_mode()?;
        execute!(io::stdout(), EnterAlternateScreen, EnableMouseCapture)?;

        Ok(())
    }
}
