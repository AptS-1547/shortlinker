//! Input validation logic

use super::state::{App, CurrentScreen, EditingField};
use crate::interfaces::tui::constants::MAX_SHORT_CODE_LENGTH;
use crate::utils::TimeParser;
use crate::utils::url_validator::validate_url;

impl App {
    /// Validate current input and update validation_errors
    pub fn validate_inputs(&mut self) {
        self.form.clear_errors();

        // Validate short code
        if !self.form.short_code.is_empty() {
            if self.form.short_code.len() > MAX_SHORT_CODE_LENGTH {
                self.form.set_error(
                    EditingField::ShortCode,
                    format!("Code too long (max {} chars)", MAX_SHORT_CODE_LENGTH),
                );
            }
            // Check for invalid characters: [a-zA-Z0-9_.-/]
            if !self
                .form
                .short_code
                .chars()
                .all(|c| c.is_alphanumeric() || matches!(c, '-' | '_' | '.' | '/'))
            {
                self.form.set_error(
                    EditingField::ShortCode,
                    "Only alphanumeric, dash, underscore, dot and slash allowed".to_string(),
                );
            }
        }

        // Validate URL
        if !self.form.target_url.is_empty() {
            if let Err(e) = validate_url(&self.form.target_url) {
                self.form.set_error(EditingField::TargetUrl, e.to_string());
            }
        } else if matches!(
            self.current_screen,
            CurrentScreen::AddLink | CurrentScreen::EditLink
        ) {
            self.form
                .set_error(EditingField::TargetUrl, "URL is required".to_string());
        }

        // Validate expire time format
        if !self.form.expire_time.is_empty()
            && let Err(e) = TimeParser::parse_expire_time(&self.form.expire_time)
        {
            self.form
                .set_error(EditingField::ExpireTime, format!("Invalid format: {}", e));
        }
    }

    /// Check if current form has any validation errors
    pub fn has_validation_errors(&self) -> bool {
        self.form.has_errors()
    }
}
