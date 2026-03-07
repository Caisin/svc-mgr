//! Service management operations for TUI

use crate::{ServiceLabel, ServiceManager, TypedServiceManager};

use super::app::App;

impl App {
    pub fn start_service(&mut self, label: &str) {
        match label.parse::<ServiceLabel>() {
            Ok(label_parsed) => {
                match TypedServiceManager::native() {
                    Ok(manager) => match manager.start(&label_parsed) {
                        Ok(action) => match action.exec() {
                            Ok(_) => {
                                self.status_message = format!("Started service: {}", label);
                                self.refresh_services();
                            }
                            Err(e) => {
                                self.status_message = format!("Error starting service: {}", e);
                            }
                        },
                        Err(e) => {
                            self.status_message = format!("Error creating start action: {}", e);
                        }
                    },
                    Err(e) => {
                        self.status_message = format!("Error initializing manager: {}", e);
                    }
                }
            }
            Err(e) => {
                self.status_message = format!("Invalid service label: {}", e);
            }
        }
    }

    pub fn stop_service(&mut self, label: &str) {
        match label.parse::<ServiceLabel>() {
            Ok(label_parsed) => {
                match TypedServiceManager::native() {
                    Ok(manager) => match manager.stop(&label_parsed) {
                        Ok(action) => match action.exec() {
                            Ok(_) => {
                                self.status_message = format!("Stopped service: {}", label);
                                self.refresh_services();
                            }
                            Err(e) => {
                                self.status_message = format!("Error stopping service: {}", e);
                            }
                        },
                        Err(e) => {
                            self.status_message = format!("Error creating stop action: {}", e);
                        }
                    },
                    Err(e) => {
                        self.status_message = format!("Error initializing manager: {}", e);
                    }
                }
            }
            Err(e) => {
                self.status_message = format!("Invalid service label: {}", e);
            }
        }
    }

    pub fn restart_service(&mut self, label: &str) {
        match label.parse::<ServiceLabel>() {
            Ok(label_parsed) => {
                match TypedServiceManager::native() {
                    Ok(manager) => match manager.restart(&label_parsed) {
                        Ok(action) => match action.exec() {
                            Ok(_) => {
                                self.status_message = format!("Restarted service: {}", label);
                                self.refresh_services();
                            }
                            Err(e) => {
                                self.status_message = format!("Error restarting service: {}", e);
                            }
                        },
                        Err(e) => {
                            self.status_message = format!("Error creating restart action: {}", e);
                        }
                    },
                    Err(e) => {
                        self.status_message = format!("Error initializing manager: {}", e);
                    }
                }
            }
            Err(e) => {
                self.status_message = format!("Invalid service label: {}", e);
            }
        }
    }

    pub fn check_service_status(&mut self, label: &str) {
        match label.parse::<ServiceLabel>() {
            Ok(label_parsed) => {
                match TypedServiceManager::native() {
                    Ok(manager) => match manager.status(&label_parsed) {
                        Ok(action) => match action.exec() {
                            Ok(output) => match output.into_status() {
                                Ok(status) => {
                                    self.status_message = format!("Service {} status: {:?}", label, status);
                                }
                                Err(e) => {
                                    self.status_message = format!("Error parsing status: {}", e);
                                }
                            },
                            Err(e) => {
                                self.status_message = format!("Error checking status: {}", e);
                            }
                        },
                        Err(e) => {
                            self.status_message = format!("Error creating status action: {}", e);
                        }
                    },
                    Err(e) => {
                        self.status_message = format!("Error initializing manager: {}", e);
                    }
                }
            }
            Err(e) => {
                self.status_message = format!("Invalid service label: {}", e);
            }
        }
    }

    pub fn view_service_info(&mut self, label: &str) {
        match label.parse::<ServiceLabel>() {
            Ok(label_parsed) => {
                match TypedServiceManager::native() {
                    Ok(manager) => match manager.info(&label_parsed) {
                        Ok(action) => match action.exec() {
                            Ok(output) => match output.into_info() {
                                Ok(info) => {
                                    self.info_content = format!(
                                        "Service: {}\nConfig Path: {}\n\n{}",
                                        info.label, info.config_path, info.config_content
                                    );
                                    self.info_scroll = 0;
                                    self.mode = super::app::Mode::ViewInfo;
                                    self.status_message = "↑↓: scroll | Esc: close".to_string();
                                }
                                Err(e) => {
                                    self.status_message = format!("Error parsing info: {}", e);
                                }
                            },
                            Err(e) => {
                                self.status_message = format!("Error getting info: {}", e);
                            }
                        },
                        Err(e) => {
                            self.status_message = format!("Error creating info action: {}", e);
                        }
                    },
                    Err(e) => {
                        self.status_message = format!("Error initializing manager: {}", e);
                    }
                }
            }
            Err(e) => {
                self.status_message = format!("Invalid service label: {}", e);
            }
        }
    }

    pub fn uninstall_service(&mut self, label: &str) {
        match label.parse::<ServiceLabel>() {
            Ok(label_parsed) => {
                match TypedServiceManager::native() {
                    Ok(manager) => match manager.uninstall(&label_parsed) {
                        Ok(action) => match action.exec() {
                            Ok(_) => {
                                self.status_message = format!("Uninstalled service: {}", label);
                                self.refresh_services();
                            }
                            Err(e) => {
                                self.status_message = format!("Error uninstalling service: {}", e);
                            }
                        },
                        Err(e) => {
                            self.status_message = format!("Error creating uninstall action: {}", e);
                        }
                    },
                    Err(e) => {
                        self.status_message = format!("Error initializing manager: {}", e);
                    }
                }
            }
            Err(e) => {
                self.status_message = format!("Invalid service label: {}", e);
            }
        }
    }
}
