//! Input handling utilities
//!
//! Provides unified input handling for text fields across different screens

use super::app::App;

/// Handle text character input
pub fn handle_text_input(app: &mut App, c: char) {
    if app.form.currently_editing.is_some() {
        app.form.push_char(c);
        // Trigger real-time validation
        app.validate_inputs();
    }
}

/// Handle backspace input
pub fn handle_backspace(app: &mut App) {
    if app.form.currently_editing.is_some() {
        app.form.pop_char();
        // Trigger real-time validation
        app.validate_inputs();
    }
}

/// Handle tab key for field navigation
pub fn handle_tab_navigation(app: &mut App) {
    app.toggle_editing();
}

/// Handle space key for toggles (currently used for force_overwrite checkbox)
pub fn handle_space_toggle(app: &mut App) {
    app.form.toggle_overwrite();
}
