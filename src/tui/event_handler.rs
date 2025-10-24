//! Event handling for TUI
//!
//! Handles keyboard events and delegates to appropriate handlers

use ratatui::crossterm::event::KeyCode;

use super::app::{App, CurrentScreen, CurrentlyEditing};
use super::input_handler::{handle_backspace, handle_space_toggle, handle_tab_navigation, handle_text_input};

/// Handle keyboard input based on current screen
pub async fn handle_key_event(app: &mut App, key_code: KeyCode) -> std::io::Result<bool> {
    match app.current_screen {
        CurrentScreen::Main => handle_main_screen(app, key_code),
        CurrentScreen::AddLink => handle_add_link_screen(app, key_code).await,
        CurrentScreen::EditLink => handle_edit_link_screen(app, key_code).await,
        CurrentScreen::DeleteConfirm => handle_delete_confirm_screen(app, key_code).await,
        CurrentScreen::ExportImport => handle_export_import_screen(app, key_code).await,
        CurrentScreen::Exiting => handle_exiting_screen(app, key_code),
        CurrentScreen::Search => handle_search_screen(app, key_code),
        CurrentScreen::Help => handle_help_screen(app, key_code),
        CurrentScreen::ViewDetails => handle_view_details_screen(app, key_code),
        CurrentScreen::FileBrowser => handle_file_browser_screen(app, key_code).await,
        CurrentScreen::ExportFileName => handle_export_filename_screen(app, key_code).await,
    }
}

/// Handle main screen input
fn handle_main_screen(app: &mut App, key_code: KeyCode) -> std::io::Result<bool> {
    match key_code {
        KeyCode::Up | KeyCode::Char('k') | KeyCode::Char('K') => app.move_selection_up(),
        KeyCode::Down | KeyCode::Char('j') | KeyCode::Char('J') => app.move_selection_down(),
        KeyCode::Home | KeyCode::Char('g') => app.jump_to_top(),
        KeyCode::End | KeyCode::Char('G') => app.jump_to_bottom(),
        KeyCode::PageUp => app.page_up(),
        KeyCode::PageDown => app.page_down(),
        KeyCode::Esc => {
            // Clear search if active
            if app.is_searching {
                app.search_input.clear();
                app.is_searching = false;
                app.filtered_links.clear();
                app.selected_index = 0;
            }
        }
        KeyCode::Char('/') => {
            app.current_screen = CurrentScreen::Search;
            app.search_input.clear();
            app.is_searching = false;
        }
        KeyCode::Char('?') | KeyCode::Char('h') | KeyCode::Char('H') => {
            app.current_screen = CurrentScreen::Help;
        }
        KeyCode::Enter | KeyCode::Char('v') | KeyCode::Char('V') => {
            if !app.links.is_empty() {
                app.current_screen = CurrentScreen::ViewDetails;
            }
        }
        KeyCode::Char('a') | KeyCode::Char('A') => {
            app.current_screen = CurrentScreen::AddLink;
            app.currently_editing = Some(CurrentlyEditing::ShortCode);
            app.clear_inputs();
        }
        KeyCode::Char('e') | KeyCode::Char('E') => {
            if !app.links.is_empty() {
                app.current_screen = CurrentScreen::EditLink;
                app.currently_editing = Some(CurrentlyEditing::TargetUrl);
                app.clear_inputs();
            }
        }
        KeyCode::Char('d') | KeyCode::Char('D') => {
            if !app.links.is_empty() {
                app.current_screen = CurrentScreen::DeleteConfirm;
            }
        }
        KeyCode::Char('x') | KeyCode::Char('X') => {
            app.current_screen = CurrentScreen::ExportImport;
        }
        KeyCode::Char('q') | KeyCode::Char('Q') => {
            app.current_screen = CurrentScreen::Exiting;
        }
        _ => {}
    }
    Ok(false)
}

/// Handle add link screen input
async fn handle_add_link_screen(app: &mut App, key_code: KeyCode) -> std::io::Result<bool> {
    match key_code {
        KeyCode::Enter => {
            if let Err(e) = app.save_new_link().await {
                app.set_error(format!("Failed to save link: {}", e));
            } else {
                app.set_status("Link added successfully!".to_string());
                app.current_screen = CurrentScreen::Main;
                if let Err(e) = app.refresh_links().await {
                    app.set_error(format!("Failed to refresh links: {}", e));
                }
            }
        }
        KeyCode::Backspace => handle_backspace(app),
        KeyCode::Esc => {
            app.current_screen = CurrentScreen::Main;
            app.clear_inputs();
        }
        KeyCode::Tab => handle_tab_navigation(app),
        KeyCode::Char(' ') => handle_space_toggle(app),
        KeyCode::Char(c) => handle_text_input(app, c),
        _ => {}
    }
    Ok(false)
}

/// Handle edit link screen input
async fn handle_edit_link_screen(app: &mut App, key_code: KeyCode) -> std::io::Result<bool> {
    match key_code {
        KeyCode::Enter => {
            if let Err(e) = app.update_selected_link().await {
                app.set_error(format!("Failed to update link: {}", e));
            } else {
                app.set_status("Link updated successfully!".to_string());
                app.current_screen = CurrentScreen::Main;
                if let Err(e) = app.refresh_links().await {
                    app.set_error(format!("Failed to refresh links: {}", e));
                }
            }
        }
        KeyCode::Backspace => {
            // Only handle backspace for editable fields (not ShortCode)
            if !matches!(app.currently_editing, Some(CurrentlyEditing::ShortCode)) {
                handle_backspace(app);
            }
        }
        KeyCode::Esc => {
            app.current_screen = CurrentScreen::Main;
            app.clear_inputs();
        }
        KeyCode::Tab => handle_tab_navigation(app),
        KeyCode::Char(c) => {
            // Only handle input for editable fields (not ShortCode)
            if !matches!(app.currently_editing, Some(CurrentlyEditing::ShortCode)) {
                handle_text_input(app, c);
            }
        }
        _ => {}
    }
    Ok(false)
}

/// Handle delete confirmation screen input
async fn handle_delete_confirm_screen(app: &mut App, key_code: KeyCode) -> std::io::Result<bool> {
    match key_code {
        KeyCode::Char('y') | KeyCode::Char('Y') => {
            if let Err(e) = app.delete_selected_link().await {
                app.set_error(format!("Failed to delete link: {}", e));
            } else {
                app.set_status("Link deleted successfully!".to_string());
                if let Err(e) = app.refresh_links().await {
                    app.set_error(format!("Failed to refresh links: {}", e));
                }
            }
            app.current_screen = CurrentScreen::Main;
        }
        KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
            app.current_screen = CurrentScreen::Main;
        }
        _ => {}
    }
    Ok(false)
}

/// Handle export/import screen input
async fn handle_export_import_screen(app: &mut App, key_code: KeyCode) -> std::io::Result<bool> {
    match key_code {
        KeyCode::Char('e') | KeyCode::Char('E') => {
            // 进入导出文件名输入界面
            app.current_screen = CurrentScreen::ExportFileName;
            app.export_filename_input = format!(
                "shortlinks_export_{}.json",
                chrono::Local::now().format("%Y%m%d_%H%M%S")
            );
        }
        KeyCode::Char('i') | KeyCode::Char('I') => {
            // 进入文件浏览器选择导入文件
            if let Err(e) = app.load_directory() {
                app.set_error(format!("Failed to load directory: {}", e));
            } else {
                app.current_screen = CurrentScreen::FileBrowser;
            }
        }
        KeyCode::Esc => {
            app.current_screen = CurrentScreen::Main;
        }
        _ => {}
    }
    Ok(false)
}

/// Handle exiting confirmation screen input
fn handle_exiting_screen(app: &mut App, key_code: KeyCode) -> std::io::Result<bool> {
    match key_code {
        KeyCode::Char('y') | KeyCode::Char('Y') => Ok(true), // Signal to exit
        KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
            app.current_screen = CurrentScreen::Main;
            Ok(false)
        }
        _ => Ok(false),
    }
}

/// Handle search screen input
fn handle_search_screen(app: &mut App, key_code: KeyCode) -> std::io::Result<bool> {
    match key_code {
        KeyCode::Esc => {
            app.current_screen = CurrentScreen::Main;
            app.search_input.clear();
            app.is_searching = false;
        }
        KeyCode::Enter => {
            // Apply search and return to main
            app.filter_links();
            app.current_screen = CurrentScreen::Main;
        }
        KeyCode::Backspace => {
            app.search_input.pop();
            app.filter_links();
        }
        KeyCode::Char(c) => {
            app.search_input.push(c);
            app.filter_links();
        }
        _ => {}
    }
    Ok(false)
}

/// Handle help screen input
fn handle_help_screen(app: &mut App, key_code: KeyCode) -> std::io::Result<bool> {
    match key_code {
        KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Char('?') | KeyCode::Char('h') | KeyCode::Char('H') => {
            app.current_screen = CurrentScreen::Main;
        }
        _ => {}
    }
    Ok(false)
}

/// Handle view details screen input
fn handle_view_details_screen(app: &mut App, key_code: KeyCode) -> std::io::Result<bool> {
    match key_code {
        KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('Q') => {
            app.current_screen = CurrentScreen::Main;
        }
        _ => {}
    }
    Ok(false)
}

/// Handle file browser screen input
async fn handle_file_browser_screen(app: &mut App, key_code: KeyCode) -> std::io::Result<bool> {
    match key_code {
        KeyCode::Up | KeyCode::Char('k') | KeyCode::Char('K') => {
            app.browser_move_up();
        }
        KeyCode::Down | KeyCode::Char('j') | KeyCode::Char('J') => {
            app.browser_move_down();
        }
        KeyCode::Enter => {
            // 尝试进入目录或选择文件
            match app.browser_navigate() {
                Ok(Some(file_path)) => {
                    // 文件被选中，执行导入
                    app.import_path = file_path.to_string_lossy().to_string();
                    if let Err(e) = app.import_links().await {
                        app.set_error(format!("Failed to import links: {}", e));
                    } else {
                        app.set_status("Links imported successfully!".to_string());
                        if let Err(e) = app.refresh_links().await {
                            app.set_error(format!("Failed to refresh links: {}", e));
                        }
                    }
                    app.current_screen = CurrentScreen::Main;
                }
                Ok(None) => {
                    // 进入了新目录，继续浏览
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
async fn handle_export_filename_screen(app: &mut App, key_code: KeyCode) -> std::io::Result<bool> {
    match key_code {
        KeyCode::Enter => {
            // 确认文件名并导出
            if app.export_filename_input.is_empty() {
                app.set_error("Filename cannot be empty".to_string());
            } else {
                // 确保文件名以 .json 结尾
                let filename = if app.export_filename_input.ends_with(".json") {
                    app.export_filename_input.clone()
                } else {
                    format!("{}.json", app.export_filename_input)
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
