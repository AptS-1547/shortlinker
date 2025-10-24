//! Event handlers for link-related screens
//!
//! Handles: Main, AddLink, EditLink, DeleteConfirm, ViewDetails

use ratatui::crossterm::event::KeyCode;

use crate::tui::app::{App, CurrentScreen, CurrentlyEditing};
use crate::tui::input_handler::{
    handle_backspace, handle_space_toggle, handle_tab_navigation, handle_text_input,
};

/// Handle main screen input
pub fn handle_main_screen(app: &mut App, key_code: KeyCode) -> std::io::Result<bool> {
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
