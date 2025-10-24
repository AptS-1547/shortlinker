//! Event handlers for miscellaneous screens
//!
//! Handles: Search, Help, Exiting

use ratatui::crossterm::event::KeyCode;

use crate::tui::app::{App, CurrentScreen};

/// Handle search screen input
pub fn handle_search_screen(app: &mut App, key_code: KeyCode) -> std::io::Result<bool> {
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
pub fn handle_help_screen(app: &mut App, key_code: KeyCode) -> std::io::Result<bool> {
    match key_code {
        KeyCode::Esc
        | KeyCode::Char('q')
        | KeyCode::Char('Q')
        | KeyCode::Char('?')
        | KeyCode::Char('h')
        | KeyCode::Char('H') => {
            app.current_screen = CurrentScreen::Main;
        }
        _ => {}
    }
    Ok(false)
}

/// Handle exiting confirmation screen input
pub fn handle_exiting_screen(app: &mut App, key_code: KeyCode) -> std::io::Result<bool> {
    match key_code {
        KeyCode::Char('y') | KeyCode::Char('Y') => Ok(true), // Signal to exit
        KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
            app.current_screen = CurrentScreen::Main;
            Ok(false)
        }
        _ => Ok(false),
    }
}
