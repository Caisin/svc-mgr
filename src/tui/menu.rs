//! Menu operations for TUI

use super::app::{App, Mode, Tab};

impl App {
    pub fn enter_menu_mode(&mut self) {
        if self.tab == Tab::Services
            && self.service_list_state.selected().is_some() {
                self.mode = Mode::Menu;
                self.menu_state.select(Some(0));
                self.status_message = "Select action: ↑↓ to navigate, Enter to confirm, Esc to cancel".to_string();
            }
    }

    pub fn exit_menu_mode(&mut self) {
        self.mode = Mode::Normal;
        self.menu_state.select(None);
        self.status_message = "Tab: switch | /: search | Enter: menu | i: info/add | e: edit | q: quit".to_string();
    }

    pub fn menu_next(&mut self) {
        let menu_items = self.get_menu_items();
        if menu_items.is_empty() {
            return;
        }
        let i = match self.menu_state.selected() {
            Some(i) => {
                if i >= menu_items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.menu_state.select(Some(i));
    }

    pub fn menu_previous(&mut self) {
        let menu_items = self.get_menu_items();
        if menu_items.is_empty() {
            return;
        }
        let i = match self.menu_state.selected() {
            Some(i) => {
                if i == 0 {
                    menu_items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.menu_state.select(Some(i));
    }

    pub fn get_menu_items(&self) -> Vec<&str> {
        vec!["Start", "Stop", "Restart", "Status", "Info", "Edit", "Uninstall"]
    }

    pub fn execute_menu_action(&mut self) {
        if let Some(menu_idx) = self.menu_state.selected()
            && let Some(service_idx) = self.service_list_state.selected() {
                let services = self.filtered_services();
                if let Some(service) = services.get(service_idx) {
                    let menu_items = self.get_menu_items();
                    if let Some(&action) = menu_items.get(menu_idx) {
                        match action {
                            "Start" => self.start_service(service),
                            "Stop" => self.stop_service(service),
                            "Restart" => self.restart_service(service),
                            "Status" => self.check_service_status(service),
                            "Info" => self.view_service_info(service),
                            "Edit" => {
                                self.exit_menu_mode();
                                if let Err(e) = self.edit_service(service) {
                                    self.status_message = format!("Error editing service: {}", e);
                                }
                                return;
                            }
                            "Uninstall" => self.uninstall_service(service),
                            _ => {}
                        }
                    }
                }
            }
        self.exit_menu_mode();
    }
}
