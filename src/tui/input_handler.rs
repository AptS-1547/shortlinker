//! Input handling utilities
//!
//! Provides unified input handling for text fields across different screens

use super::app::{App, CurrentlyEditing};

/// Handle text character input
pub fn handle_text_input(app: &mut App, c: char) {
    if let Some(editing) = &app.currently_editing {
        match editing {
            CurrentlyEditing::ShortCode => app.short_code_input.push(c),
            CurrentlyEditing::TargetUrl => app.target_url_input.push(c),
            CurrentlyEditing::ExpireTime => app.expire_time_input.push(c),
            CurrentlyEditing::Password => app.password_input.push(c),
        }
        // Trigger real-time validation
        app.validate_inputs();
    }
}

/// Handle backspace input
pub fn handle_backspace(app: &mut App) {
    if let Some(editing) = &app.currently_editing {
        match editing {
            CurrentlyEditing::ShortCode => {
                app.short_code_input.pop();
            }
            CurrentlyEditing::TargetUrl => {
                app.target_url_input.pop();
            }
            CurrentlyEditing::ExpireTime => {
                app.expire_time_input.pop();
            }
            CurrentlyEditing::Password => {
                app.password_input.pop();
            }
        }
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
    if matches!(app.currently_editing, Some(CurrentlyEditing::ShortCode)) {
        app.force_overwrite = !app.force_overwrite;
    }
}
