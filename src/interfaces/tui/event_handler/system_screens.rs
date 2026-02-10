//! Event handlers for system operation screens
//!
//! Handles: SystemMenu, ServerStatus, ConfigList, ConfigEdit,
//!          ConfigResetConfirm, PasswordReset, ImportModeSelect

use ratatui::crossterm::event::KeyCode;

use crate::interfaces::tui::app::{App, CurrentScreen, PasswordField};

/// Handle system menu screen input
pub async fn handle_system_menu_screen(app: &mut App, key_code: KeyCode) -> std::io::Result<bool> {
    match key_code {
        KeyCode::Char('s') | KeyCode::Char('S') => {
            app.fetch_server_status().await;
            app.current_screen = CurrentScreen::ServerStatus;
        }
        KeyCode::Char('c') | KeyCode::Char('C') => {
            app.fetch_configs().await;
            app.current_screen = CurrentScreen::ConfigList;
        }
        KeyCode::Char('p') | KeyCode::Char('P') => {
            app.system.password_input.clear();
            app.system.password_confirm.clear();
            app.system.password_error = None;
            app.system.password_field = Some(PasswordField::Password);
            app.current_screen = CurrentScreen::PasswordReset;
        }
        KeyCode::Esc => {
            app.current_screen = CurrentScreen::Main;
        }
        _ => {}
    }
    Ok(false)
}

/// Handle server status screen input
pub async fn handle_server_status_screen(
    app: &mut App,
    key_code: KeyCode,
) -> std::io::Result<bool> {
    match key_code {
        KeyCode::Char('r') | KeyCode::Char('R') => {
            app.fetch_server_status().await;
        }
        KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('Q') => {
            app.current_screen = CurrentScreen::SystemMenu;
        }
        _ => {}
    }
    Ok(false)
}

/// Handle config list screen input
pub async fn handle_config_list_screen(app: &mut App, key_code: KeyCode) -> std::io::Result<bool> {
    match key_code {
        KeyCode::Up | KeyCode::Char('k') | KeyCode::Char('K') => {
            app.config_move_up();
        }
        KeyCode::Down | KeyCode::Char('j') | KeyCode::Char('J') => {
            app.config_move_down();
        }
        KeyCode::Char('e') | KeyCode::Char('E') | KeyCode::Enter => {
            if let Some(cfg) = app.get_selected_config() {
                if cfg.sensitive {
                    app.set_error("Sensitive configs cannot be edited in TUI".to_string());
                } else if !cfg.editable {
                    app.set_error("This config is read-only".to_string());
                } else {
                    let key = cfg.key.clone();
                    let value = cfg.value.clone();
                    app.system.config_edit_key = key;
                    app.system.config_edit_value = value;
                    app.system.config_edit_error = None;
                    app.current_screen = CurrentScreen::ConfigEdit;
                }
            }
        }
        KeyCode::Char('r') | KeyCode::Char('R') => {
            if let Some(cfg) = app.get_selected_config() {
                if !cfg.editable {
                    app.set_error("This config is read-only".to_string());
                } else {
                    let key = cfg.key.clone();
                    app.system.config_edit_key = key;
                    app.current_screen = CurrentScreen::ConfigResetConfirm;
                }
            }
        }
        KeyCode::Esc => {
            app.current_screen = CurrentScreen::SystemMenu;
        }
        _ => {}
    }
    Ok(false)
}

/// Handle config edit screen input
pub async fn handle_config_edit_screen(app: &mut App, key_code: KeyCode) -> std::io::Result<bool> {
    match key_code {
        KeyCode::Enter => match app.update_config().await {
            Ok(()) => {
                app.set_status(format!(
                    "Config '{}' updated successfully",
                    app.system.config_edit_key
                ));
                app.fetch_configs().await;
                app.current_screen = CurrentScreen::ConfigList;
            }
            Err(e) => {
                app.system.config_edit_error = Some(e);
            }
        },
        KeyCode::Backspace => {
            app.system.config_edit_value.pop();
        }
        KeyCode::Esc => {
            app.current_screen = CurrentScreen::ConfigList;
        }
        KeyCode::Char(c) => {
            app.system.config_edit_value.push(c);
        }
        _ => {}
    }
    Ok(false)
}

/// Handle config reset confirmation screen input
pub async fn handle_config_reset_confirm_screen(
    app: &mut App,
    key_code: KeyCode,
) -> std::io::Result<bool> {
    match key_code {
        KeyCode::Char('y') | KeyCode::Char('Y') => {
            match app.reset_config().await {
                Ok(()) => {
                    app.set_status(format!(
                        "Config '{}' reset to default",
                        app.system.config_edit_key
                    ));
                    app.fetch_configs().await;
                }
                Err(e) => {
                    app.set_error(e);
                }
            }
            app.current_screen = CurrentScreen::ConfigList;
        }
        KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
            app.current_screen = CurrentScreen::ConfigList;
        }
        _ => {}
    }
    Ok(false)
}

/// Handle password reset screen input
pub async fn handle_password_reset_screen(
    app: &mut App,
    key_code: KeyCode,
) -> std::io::Result<bool> {
    match key_code {
        KeyCode::Tab => {
            // Toggle between Password and Confirm fields
            app.system.password_field = Some(match app.system.password_field {
                Some(PasswordField::Password) => PasswordField::Confirm,
                Some(PasswordField::Confirm) => PasswordField::Password,
                None => PasswordField::Password,
            });
        }
        KeyCode::Enter => match app.reset_admin_password().await {
            Ok(()) => {
                app.set_status("Admin password reset successfully".to_string());
                app.system.password_input.clear();
                app.system.password_confirm.clear();
                app.system.password_error = None;
                app.system.password_field = None;
                app.current_screen = CurrentScreen::SystemMenu;
            }
            Err(e) => {
                app.system.password_error = Some(e);
            }
        },
        KeyCode::Backspace => match app.system.password_field {
            Some(PasswordField::Password) => {
                app.system.password_input.pop();
            }
            Some(PasswordField::Confirm) => {
                app.system.password_confirm.pop();
            }
            None => {}
        },
        KeyCode::Esc => {
            app.system.password_input.clear();
            app.system.password_confirm.clear();
            app.system.password_error = None;
            app.system.password_field = None;
            app.current_screen = CurrentScreen::SystemMenu;
        }
        KeyCode::Char(c) => match app.system.password_field {
            Some(PasswordField::Password) => {
                app.system.password_input.push(c);
            }
            Some(PasswordField::Confirm) => {
                app.system.password_confirm.push(c);
            }
            None => {}
        },
        _ => {}
    }
    Ok(false)
}

/// Handle import mode selection screen input
pub async fn handle_import_mode_screen(app: &mut App, key_code: KeyCode) -> std::io::Result<bool> {
    match key_code {
        KeyCode::Char('s') | KeyCode::Char('S') => {
            app.system.import_overwrite = false;
            if let Err(e) = app.load_directory() {
                app.set_error(format!("Failed to load directory: {}", e));
            } else {
                app.current_screen = CurrentScreen::FileBrowser;
            }
        }
        KeyCode::Char('o') | KeyCode::Char('O') => {
            app.system.import_overwrite = true;
            if let Err(e) = app.load_directory() {
                app.set_error(format!("Failed to load directory: {}", e));
            } else {
                app.current_screen = CurrentScreen::FileBrowser;
            }
        }
        KeyCode::Esc => {
            app.current_screen = CurrentScreen::ExportImport;
        }
        _ => {}
    }
    Ok(false)
}
