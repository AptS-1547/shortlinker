//! Event handlers for link-related screens
//!
//! Handles: Main, AddLink, EditLink, DeleteConfirm, ViewDetails

use ratatui::crossterm::event::KeyCode;

use crate::interfaces::tui::app::{App, CurrentScreen, EditingField};
use crate::interfaces::tui::input_handler::{
    handle_backspace, handle_space_toggle, handle_tab_navigation, handle_text_input,
};

/// Handle main screen input
pub async fn handle_main_screen(app: &mut App, key_code: KeyCode) -> std::io::Result<bool> {
    match key_code {
        KeyCode::Up | KeyCode::Char('k') | KeyCode::Char('K') => app.move_selection_up(),
        KeyCode::Down | KeyCode::Char('j') | KeyCode::Char('J') => app.move_selection_down(),
        KeyCode::Home | KeyCode::Char('g') => {
            if let Err(e) = app.first_page().await {
                app.set_error(format!("Failed to load page: {}", e));
            }
        }
        KeyCode::End | KeyCode::Char('G') => {
            if let Err(e) = app.last_page().await {
                app.set_error(format!("Failed to load page: {}", e));
            }
        }
        KeyCode::PageUp => app.page_up(),
        KeyCode::PageDown => app.page_down(),
        // 翻页：[ 上一页，] 下一页
        KeyCode::Char('[') => {
            if let Err(e) = app.prev_page().await {
                app.set_error(format!("Failed to load page: {}", e));
            }
        }
        KeyCode::Char(']') => {
            if let Err(e) = app.next_page().await {
                app.set_error(format!("Failed to load page: {}", e));
            }
        }
        KeyCode::Esc => {
            // Clear search if active, then clear selection
            if app.is_searching {
                if let Err(e) = app.clear_search().await {
                    app.set_error(format!("Failed to clear search: {}", e));
                }
            } else if !app.selected_items.is_empty() {
                app.clear_selection();
            }
        }
        KeyCode::Char('/') => {
            app.inline_search_mode = true;
            app.search_input.clear();
        }
        KeyCode::Char('?') | KeyCode::Char('h') | KeyCode::Char('H') => {
            app.current_screen = CurrentScreen::Help;
        }
        KeyCode::Enter | KeyCode::Char('v') | KeyCode::Char('V') => {
            if !app.page_links.is_empty() {
                app.current_screen = CurrentScreen::ViewDetails;
            }
        }
        KeyCode::Char('a') | KeyCode::Char('A') => {
            app.current_screen = CurrentScreen::AddLink;
            app.form.currently_editing = Some(EditingField::ShortCode);
            app.clear_inputs();
        }
        KeyCode::Char('e') | KeyCode::Char('E') => {
            if !app.page_links.is_empty() {
                app.current_screen = CurrentScreen::EditLink;
                app.form.currently_editing = Some(EditingField::TargetUrl);
                app.clear_inputs();
            }
        }
        KeyCode::Char('d') | KeyCode::Char('D') => {
            if !app.selected_items.is_empty() {
                app.current_screen = CurrentScreen::BatchDeleteConfirm;
            } else if !app.page_links.is_empty() {
                app.current_screen = CurrentScreen::DeleteConfirm;
            }
        }
        KeyCode::Char('x') | KeyCode::Char('X') => {
            app.current_screen = CurrentScreen::ExportImport;
        }
        KeyCode::Char('o') | KeyCode::Char('O') => {
            app.current_screen = CurrentScreen::SystemMenu;
        }
        KeyCode::Char('q') | KeyCode::Char('Q') => {
            app.current_screen = CurrentScreen::Exiting;
        }
        // Sorting
        KeyCode::Char('s') => {
            app.cycle_sort_column();
            if let Some(col) = app.sort_column {
                let col_name = match col {
                    crate::interfaces::tui::app::SortColumn::Code => "Code",
                    crate::interfaces::tui::app::SortColumn::Url => "URL",
                    crate::interfaces::tui::app::SortColumn::Clicks => "Clicks",
                    crate::interfaces::tui::app::SortColumn::Status => "Status",
                };
                let dir = if app.sort_ascending { "↑" } else { "↓" };
                app.set_status(format!("Sorted by {} {}", col_name, dir));
            }
        }
        KeyCode::Char('S') => {
            app.toggle_sort_direction();
            if let Some(col) = app.sort_column {
                let col_name = match col {
                    crate::interfaces::tui::app::SortColumn::Code => "Code",
                    crate::interfaces::tui::app::SortColumn::Url => "URL",
                    crate::interfaces::tui::app::SortColumn::Clicks => "Clicks",
                    crate::interfaces::tui::app::SortColumn::Status => "Status",
                };
                let dir = if app.sort_ascending { "↑" } else { "↓" };
                app.set_status(format!("Sorted by {} {}", col_name, dir));
            }
        }
        // Copy to clipboard
        KeyCode::Char('y') => {
            #[cfg(feature = "tui")]
            if let Some(link) = app.get_selected_link()
                && let Ok(mut clipboard) = arboard::Clipboard::new()
            {
                let code = link.code.clone();
                if clipboard.set_text(&code).is_ok() {
                    app.set_status(format!("Copied: {}", code));
                }
            }
        }
        KeyCode::Char('Y') => {
            #[cfg(feature = "tui")]
            if let Some(link) = app.get_selected_link()
                && let Ok(mut clipboard) = arboard::Clipboard::new()
            {
                let url = link.target.clone();
                if clipboard.set_text(&url).is_ok() {
                    app.set_status("Copied URL".to_string());
                }
            }
        }
        // Batch selection
        KeyCode::Char(' ') => {
            app.toggle_selection();
        }
        _ => {}
    }
    Ok(false)
}

/// Handle add link screen input
pub async fn handle_add_link_screen(app: &mut App, key_code: KeyCode) -> std::io::Result<bool> {
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
pub async fn handle_edit_link_screen(app: &mut App, key_code: KeyCode) -> std::io::Result<bool> {
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
            if !matches!(app.form.currently_editing, Some(EditingField::ShortCode)) {
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
            if !matches!(app.form.currently_editing, Some(EditingField::ShortCode)) {
                handle_text_input(app, c);
            }
        }
        _ => {}
    }
    Ok(false)
}

/// Handle delete confirmation screen input
pub async fn handle_delete_confirm_screen(
    app: &mut App,
    key_code: KeyCode,
) -> std::io::Result<bool> {
    match key_code {
        KeyCode::Char('y') | KeyCode::Char('Y') => {
            if let Err(e) = app.delete_selected_link().await {
                app.set_error(format!("Failed to delete link: {}", e));
            } else {
                app.set_status("Link deleted successfully!".to_string());
                if let Err(e) = app.refresh_links().await {
                    app.set_error(format!("Failed to refresh links: {}", e));
                }
                // 如果当前页为空且有上一页，自动回退
                if app.page_links.is_empty()
                    && app.has_prev_page()
                    && let Err(e) = app.prev_page().await
                {
                    app.set_error(format!("Failed to load previous page: {}", e));
                }
                app.clamp_selection();
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

/// Handle view details screen input
pub fn handle_view_details_screen(app: &mut App, key_code: KeyCode) -> std::io::Result<bool> {
    match key_code {
        KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('Q') => {
            app.current_screen = CurrentScreen::Main;
        }
        _ => {}
    }
    Ok(false)
}

/// Handle batch delete confirmation screen input
pub async fn handle_batch_delete_confirm_screen(
    app: &mut App,
    key_code: KeyCode,
) -> std::io::Result<bool> {
    match key_code {
        KeyCode::Char('y') | KeyCode::Char('Y') => {
            let codes_to_delete: Vec<String> = app.selected_items.iter().cloned().collect();
            if codes_to_delete.is_empty() {
                app.current_screen = CurrentScreen::Main;
                return Ok(false);
            }
            let total = codes_to_delete.len();

            match app.link_client.batch_delete(codes_to_delete).await {
                Ok(result) => {
                    let deleted = result.deleted.len();
                    let failed = result.errors.len();
                    let not_found = result.not_found.len();

                    app.selected_items.clear();

                    if let Err(e) = app.refresh_links().await {
                        app.set_error(format!("Failed to refresh links: {}", e));
                    } else if failed > 0 || not_found > 0 {
                        app.set_error(format!(
                            "Deleted {}/{} links ({} failed, {} not found)",
                            deleted, total, failed, not_found
                        ));
                    } else {
                        app.set_status(format!("Deleted {} links", deleted));
                    }
                    // 如果当前页为空且有上一页，自动回退
                    if app.page_links.is_empty()
                        && app.has_prev_page()
                        && let Err(e) = app.prev_page().await
                    {
                        app.set_error(format!("Failed to load previous page: {}", e));
                    }
                    app.clamp_selection();
                }
                Err(e) => {
                    app.selected_items.clear();
                    app.set_error(format!("Batch delete failed: {}", e));
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

/// Handle inline search mode input
pub async fn handle_inline_search(app: &mut App, key_code: KeyCode) -> std::io::Result<bool> {
    match key_code {
        KeyCode::Esc => {
            if app.is_searching {
                // 有活跃搜索时，清除搜索并恢复全量
                if let Err(e) = app.clear_search().await {
                    app.set_error(format!("Failed to clear search: {}", e));
                }
            } else {
                // 没有活跃搜索，只是关闭搜索栏
                app.inline_search_mode = false;
                app.search_input.clear();
            }
        }
        KeyCode::Enter => {
            app.inline_search_mode = false;
            // 提交搜索到数据库
            if let Err(e) = app.execute_search().await {
                app.set_error(format!("Search failed: {}", e));
            }
        }
        KeyCode::Backspace => {
            app.search_input.pop();
        }
        KeyCode::Up => {
            app.move_selection_up();
        }
        KeyCode::Down => {
            app.move_selection_down();
        }
        KeyCode::Char(c) => {
            app.search_input.push(c);
        }
        _ => {}
    }
    Ok(false)
}
