//! Event handling for TUI
//!
//! Handles keyboard events and delegates to appropriate handlers
//!
//! This module is organized by screen type:
//! - link_screens: Main, AddLink, EditLink, DeleteConfirm, ViewDetails
//! - file_screens: ExportImport, FileBrowser, ExportFileName
//! - misc_screens: Search, Help, Exiting

use ratatui::crossterm::event::KeyCode;

use crate::interfaces::tui::app::{App, CurrentScreen};

mod file_screens;
mod link_screens;
mod misc_screens;

use file_screens::*;
use link_screens::*;
use misc_screens::*;

/// Handle keyboard input based on current screen
pub async fn handle_key_event(app: &mut App, key_code: KeyCode) -> std::io::Result<bool> {
    match app.current_screen {
        CurrentScreen::Main => handle_main_screen(app, key_code),
        CurrentScreen::AddLink => handle_add_link_screen(app, key_code).await,
        CurrentScreen::EditLink => handle_edit_link_screen(app, key_code).await,
        CurrentScreen::DeleteConfirm => handle_delete_confirm_screen(app, key_code).await,
        CurrentScreen::ViewDetails => handle_view_details_screen(app, key_code),
        CurrentScreen::ExportImport => handle_export_import_screen(app, key_code).await,
        CurrentScreen::FileBrowser => handle_file_browser_screen(app, key_code).await,
        CurrentScreen::ExportFileName => handle_export_filename_screen(app, key_code).await,
        CurrentScreen::Search => handle_search_screen(app, key_code),
        CurrentScreen::Help => handle_help_screen(app, key_code),
        CurrentScreen::Exiting => handle_exiting_screen(app, key_code),
    }
}
