// UI submodules
mod add_link;
mod common;
mod delete_confirm;
mod edit_link;
mod exiting;
mod export_filename;
mod export_import;
mod file_browser;
mod help;
mod main_screen;
mod search;
mod view_details;

// Re-export common utilities
pub use common::{draw_footer, draw_status_bar, draw_title_bar};

// Re-export screen drawing functions
pub use add_link::draw_add_link_screen;
pub use delete_confirm::draw_delete_confirm_screen;
pub use edit_link::draw_edit_link_screen;
pub use exiting::draw_exiting_screen;
pub use export_filename::draw_export_filename_screen;
pub use export_import::draw_export_import_screen;
pub use file_browser::draw_file_browser_screen;
pub use help::draw_help_screen;
pub use main_screen::draw_main_screen;
pub use search::draw_search_screen;
pub use view_details::draw_view_details_screen;

use super::app::{App, CurrentScreen};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
};

/// Main UI rendering entry point
pub fn ui(frame: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Min(10),   // Main content
            Constraint::Length(3), // Status
            Constraint::Length(2), // Footer
        ])
        .split(frame.area());

    // Enhanced title with version and stats
    draw_title_bar(frame, app, chunks[0]);

    // Main content based on current screen
    match app.current_screen {
        CurrentScreen::Main => draw_main_screen(frame, app, chunks[1]),
        CurrentScreen::AddLink => draw_add_link_screen(frame, app, chunks[1]),
        CurrentScreen::EditLink => draw_edit_link_screen(frame, app, chunks[1]),
        CurrentScreen::DeleteConfirm => draw_delete_confirm_screen(frame, app, chunks[1]),
        CurrentScreen::ExportImport => draw_export_import_screen(frame, app, chunks[1]),
        CurrentScreen::Exiting => draw_exiting_screen(frame, chunks[1]),
        CurrentScreen::Search => draw_search_screen(frame, app, chunks[1]),
        CurrentScreen::Help => draw_help_screen(frame, chunks[1]),
        CurrentScreen::ViewDetails => draw_view_details_screen(frame, app, chunks[1]),
        CurrentScreen::FileBrowser => draw_file_browser_screen(frame, app, chunks[1]),
        CurrentScreen::ExportFileName => draw_export_filename_screen(frame, app, chunks[1]),
    }

    // Enhanced status bar
    draw_status_bar(frame, app, chunks[2]);

    // Enhanced footer with styled shortcuts
    draw_footer(frame, app, chunks[3]);
}
