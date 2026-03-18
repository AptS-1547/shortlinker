//! Event handlers for file-related screens
//!
//! Handles: ExportImport, FileBrowser, ExportFileName

use ratatui::crossterm::event::KeyCode;

use crate::interfaces::tui::app::{App, CurrentScreen};

/// Handle export/import screen input
pub async fn handle_export_import_screen(
    app: &mut App,
    key_code: KeyCode,
) -> std::io::Result<bool> {
    match key_code {
        KeyCode::Char('e') | KeyCode::Char('E') => {
            app.current_screen = CurrentScreen::ExportFileName;
            app.export_filename_input = format!(
                "shortlinks_export_{}.csv",
                chrono::Local::now().format("%Y%m%d_%H%M%S")
            );
        }
        KeyCode::Char('i') | KeyCode::Char('I') => {
            app.current_screen = CurrentScreen::ImportModeSelect;
        }
        KeyCode::Esc => {
            app.current_screen = CurrentScreen::Main;
        }
        _ => {}
    }
    Ok(false)
}

/// Handle file browser screen input
pub async fn handle_file_browser_screen(app: &mut App, key_code: KeyCode) -> std::io::Result<bool> {
    match key_code {
        KeyCode::Up | KeyCode::Char('k') | KeyCode::Char('K') => {
            app.browser_move_up();
        }
        KeyCode::Down | KeyCode::Char('j') | KeyCode::Char('J') => {
            app.browser_move_down();
        }
        KeyCode::Enter => {
            match app.browser_navigate() {
                Ok(Some(file_path)) => {
                    // File selected, perform import
                    app.import_path = file_path.to_string_lossy().to_string();
                    match app.import_links(app.system.import_overwrite).await {
                        Ok(result) => {
                            let failed = result.failed_items.len();
                            let total = result.success_count + result.skipped_count + failed;
                            let import_msg = if failed > 0 {
                                format!(
                                    "Imported {}/{} links ({} failed, {} skipped)",
                                    result.success_count, total, failed, result.skipped_count
                                )
                            } else {
                                format!(
                                    "Imported {} links ({} skipped)",
                                    result.success_count, result.skipped_count
                                )
                            };
                            if let Err(e) = app.refresh_links().await {
                                app.set_error(format!("{}; refresh failed: {}", import_msg, e));
                            } else if failed > 0 {
                                app.set_error(import_msg);
                            } else {
                                app.set_status(import_msg);
                            }
                        }
                        Err(e) => {
                            app.set_error(format!("Failed to import links: {}", e));
                        }
                    }
                    app.current_screen = CurrentScreen::Main;
                }
                Ok(None) => {
                    // Entered new directory, continue browsing
                }
                Err(e) => {
                    app.set_error(format!("Navigation error: {}", e));
                }
            }
        }
        KeyCode::Esc => {
            app.current_screen = CurrentScreen::ExportImport;
        }
        _ => {}
    }
    Ok(false)
}

/// Handle export filename input screen
pub async fn handle_export_filename_screen(
    app: &mut App,
    key_code: KeyCode,
) -> std::io::Result<bool> {
    match key_code {
        KeyCode::Enter => {
            if app.export_filename_input.is_empty() {
                app.set_error("Filename cannot be empty".to_string());
            } else {
                // Ensure filename ends with .csv
                let filename = if app.export_filename_input.ends_with(".csv") {
                    app.export_filename_input.clone()
                } else {
                    format!("{}.csv", app.export_filename_input)
                };

                app.export_path = filename;
                if let Err(e) = app.export_links().await {
                    app.set_error(format!("Failed to export links: {}", e));
                } else {
                    app.set_status(format!("Links exported to: {}", app.export_path));
                }
                app.current_screen = CurrentScreen::Main;
            }
        }
        KeyCode::Backspace => {
            app.export_filename_input.pop();
        }
        KeyCode::Esc => {
            app.current_screen = CurrentScreen::ExportImport;
        }
        KeyCode::Char(c) => {
            app.export_filename_input.push(c);
        }
        _ => {}
    }
    Ok(false)
}
