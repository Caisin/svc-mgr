//! Event handling for TUI

use crossterm::event::{Event, KeyCode, KeyEventKind, KeyModifiers};

use super::app::{App, Mode, Tab};

pub fn handle_events(app: &mut App) -> Result<(), Box<dyn std::error::Error>> {
    if let Event::Key(key) = crossterm::event::read()?
        && key.kind == KeyEventKind::Press {
            match app.mode {
                Mode::Normal => handle_normal_mode(app, key.code, key.modifiers),
                Mode::Search => handle_search_mode(app, key.code),
                Mode::Edit => handle_edit_mode(app, key.code, key.modifiers),
                Mode::Menu => handle_menu_mode(app, key.code),
                Mode::AddEnv => handle_add_env_mode(app, key.code, key.modifiers),
                Mode::ViewInfo => handle_view_info_mode(app, key.code),
            }
        }
    Ok(())
}

fn handle_normal_mode(app: &mut App, code: KeyCode, _modifiers: KeyModifiers) {
    match code {
        KeyCode::Char('q') => {
            app.should_quit = true;
        }
        KeyCode::Tab => {
            app.switch_tab();
        }
        KeyCode::Down | KeyCode::Char('j') => {
            app.next_item();
        }
        KeyCode::Up | KeyCode::Char('k') => {
            app.previous_item();
        }
        KeyCode::Char('r') => match app.tab {
            Tab::Services => app.refresh_services(),
            Tab::Environment => app.refresh_env_vars(),
        },
        KeyCode::Char('/') => {
            app.enter_search_mode();
        }
        KeyCode::Char('e') => {
            app.enter_edit_mode();
        }
        KeyCode::Enter | KeyCode::Char(' ')
            if app.tab == Tab::Services => {
                app.enter_menu_mode();
            }
        KeyCode::Char('i') => match app.tab {
            Tab::Services
                if let Some(idx) = app.service_list_state.selected() => {
                    let services = app.filtered_services();
                    if let Some(service) = services.get(idx) {
                        app.view_service_info(service);
                    }
                }
            Tab::Environment => {
                app.enter_add_env_mode();
            }
            _ => {}
        },
        KeyCode::Char('d')
            if app.tab == Tab::Environment => {
                app.delete_env_var();
            }
        KeyCode::Char('s')
            if app.tab == Tab::Services
                && let Some(idx) = app.service_list_state.selected() => {
                    let services = app.filtered_services();
                    if let Some(service) = services.get(idx) {
                        app.start_service(service);
                    }
                }
        KeyCode::Char('t')
            if app.tab == Tab::Services
                && let Some(idx) = app.service_list_state.selected() => {
                    let services = app.filtered_services();
                    if let Some(service) = services.get(idx) {
                        app.stop_service(service);
                    }
                }
        _ => {}
    }
}

fn handle_search_mode(app: &mut App, code: KeyCode) {
    match code {
        KeyCode::Char(c) => {
            app.search_query.push(c);
        }
        KeyCode::Backspace => {
            app.search_query.pop();
        }
        KeyCode::Enter | KeyCode::Esc => {
            app.exit_search_mode();
        }
        _ => {}
    }
}

fn handle_edit_mode(app: &mut App, code: KeyCode, modifiers: KeyModifiers) {
    match code {
        KeyCode::Char(c) if !modifiers.contains(KeyModifiers::CONTROL) => {
            app.edit_value.push(c);
        }
        KeyCode::Backspace => {
            app.edit_value.pop();
        }
        KeyCode::Enter => {
            app.save_edit();
        }
        KeyCode::Esc => {
            app.cancel_edit();
        }
        _ => {}
    }
}

fn handle_menu_mode(app: &mut App, code: KeyCode) {
    match code {
        KeyCode::Down | KeyCode::Char('j') => {
            app.menu_next();
        }
        KeyCode::Up | KeyCode::Char('k') => {
            app.menu_previous();
        }
        KeyCode::Enter => {
            app.execute_menu_action();
        }
        KeyCode::Esc => {
            app.exit_menu_mode();
        }
        _ => {}
    }
}

fn handle_add_env_mode(app: &mut App, code: KeyCode, modifiers: KeyModifiers) {
    match code {
        KeyCode::Char(c) if !modifiers.contains(KeyModifiers::CONTROL) => {
            if app.status_message.contains("Key:") {
                app.edit_key.push(c);
            } else {
                app.edit_value.push(c);
            }
        }
        KeyCode::Backspace => {
            if app.status_message.contains("Key:") {
                app.edit_key.pop();
            } else {
                app.edit_value.pop();
            }
        }
        KeyCode::Tab
            if app.status_message.contains("Key:") && !app.edit_key.is_empty() => {
                app.switch_to_value_input();
            }
        KeyCode::Enter
            if !app.status_message.contains("Key:") => {
                app.save_new_env();
            }
        KeyCode::Esc => {
            app.cancel_edit();
        }
        _ => {}
    }
}

fn handle_view_info_mode(app: &mut App, code: KeyCode) {
    match code {
        KeyCode::Down | KeyCode::Char('j') => {
            app.scroll_info_down();
        }
        KeyCode::Up | KeyCode::Char('k') => {
            app.scroll_info_up();
        }
        KeyCode::Esc | KeyCode::Char('q') => {
            app.exit_info_mode();
        }
        _ => {}
    }
}
