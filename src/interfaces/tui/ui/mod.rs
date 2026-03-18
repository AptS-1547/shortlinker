// UI submodules
mod add_link;
mod batch_delete_confirm;
mod common;
mod config_edit;
mod config_list;
mod config_reset_confirm;
mod delete_confirm;
mod detail_panel;
mod edit_link;
mod exiting;
mod export_filename;
mod export_import;
mod file_browser;
mod help;
mod import_mode;
mod inline_search;
mod main_screen;
mod password_reset;
mod search;
mod server_status;
mod system_menu;
mod view_details;
pub mod widgets;

// Re-export common utilities
pub use common::{draw_footer, draw_status_bar, draw_title_bar};

// Re-export screen drawing functions
pub use add_link::draw_add_link_screen;
pub use batch_delete_confirm::draw_batch_delete_confirm_screen;
pub use config_edit::draw_config_edit_screen;
pub use config_list::draw_config_list_screen;
pub use config_reset_confirm::draw_config_reset_confirm_screen;
pub use delete_confirm::draw_delete_confirm_screen;
pub use detail_panel::draw_detail_panel;
pub use edit_link::draw_edit_link_screen;
pub use exiting::draw_exiting_screen;
pub use export_filename::draw_export_filename_screen;
pub use export_import::draw_export_import_screen;
pub use file_browser::draw_file_browser_screen;
pub use help::draw_help_screen;
pub use import_mode::draw_import_mode_screen;
pub use inline_search::draw_inline_search_bar;
pub use main_screen::draw_main_screen;
pub use password_reset::draw_password_reset_screen;
pub use search::draw_search_screen;
pub use server_status::draw_server_status_screen;
pub use system_menu::draw_system_menu_screen;
pub use view_details::draw_view_details_screen;

use super::app::{App, CurrentScreen};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
};

/// Main UI rendering entry point
pub fn ui(frame: &mut Frame, app: &mut App) {
    // Calculate layout based on whether inline search is active
    let main_chunks = if app.inline_search_mode {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Title
                Constraint::Min(10),   // Main content
                Constraint::Length(3), // Inline search bar
                Constraint::Length(3), // Status
                Constraint::Length(2), // Footer
            ])
            .split(frame.area())
    } else {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Title
                Constraint::Min(10),   // Main content
                Constraint::Length(3), // Status
                Constraint::Length(2), // Footer
            ])
            .split(frame.area())
    };

    // Enhanced title with version and stats
    draw_title_bar(frame, app, main_chunks[0]);

    // Main content based on current screen
    match app.current_screen {
        CurrentScreen::Main => {
            // Dual-panel layout for main screen
            let content_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Percentage(60), // Left: link list
                    Constraint::Percentage(40), // Right: detail panel
                ])
                .split(main_chunks[1]);

            draw_main_screen(frame, app, content_chunks[0]);
            draw_detail_panel(frame, app, content_chunks[1]);
        }
        CurrentScreen::AddLink => draw_add_link_screen(frame, app, main_chunks[1]),
        CurrentScreen::EditLink => draw_edit_link_screen(frame, app, main_chunks[1]),
        CurrentScreen::DeleteConfirm => draw_delete_confirm_screen(frame, app, main_chunks[1]),
        CurrentScreen::BatchDeleteConfirm => {
            draw_batch_delete_confirm_screen(frame, app, main_chunks[1])
        }
        CurrentScreen::ExportImport => draw_export_import_screen(frame, app, main_chunks[1]),
        CurrentScreen::Exiting => draw_exiting_screen(frame, main_chunks[1]),
        CurrentScreen::Search => draw_search_screen(frame, app, main_chunks[1]),
        CurrentScreen::Help => draw_help_screen(frame, main_chunks[1]),
        CurrentScreen::ViewDetails => draw_view_details_screen(frame, app, main_chunks[1]),
        CurrentScreen::FileBrowser => draw_file_browser_screen(frame, app, main_chunks[1]),
        CurrentScreen::ExportFileName => draw_export_filename_screen(frame, app, main_chunks[1]),
        CurrentScreen::SystemMenu => draw_system_menu_screen(frame, app, main_chunks[1]),
        CurrentScreen::ServerStatus => draw_server_status_screen(frame, app, main_chunks[1]),
        CurrentScreen::ConfigList => draw_config_list_screen(frame, app, main_chunks[1]),
        CurrentScreen::ConfigEdit => draw_config_edit_screen(frame, app, main_chunks[1]),
        CurrentScreen::ConfigResetConfirm => {
            draw_config_reset_confirm_screen(frame, app, main_chunks[1])
        }
        CurrentScreen::PasswordReset => draw_password_reset_screen(frame, app, main_chunks[1]),
        CurrentScreen::ImportModeSelect => draw_import_mode_screen(frame, app, main_chunks[1]),
    }

    // Inline search bar (if active)
    if app.inline_search_mode {
        draw_inline_search_bar(frame, app, main_chunks[2]);
        // Status bar and footer shift down
        draw_status_bar(frame, app, main_chunks[3]);
        draw_footer(frame, app, main_chunks[4]);
    } else {
        // Enhanced status bar
        draw_status_bar(frame, app, main_chunks[2]);
        // Enhanced footer with styled shortcuts
        draw_footer(frame, app, main_chunks[3]);
    }
}
