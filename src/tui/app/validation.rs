//! Input validation logic

use super::state::{App, CurrentScreen};
use crate::utils::TimeParser;

impl App {
    /// Validate current input and update validation_errors
    pub fn validate_inputs(&mut self) {
        self.validation_errors.clear();

        // Validate short code
        if !self.short_code_input.is_empty() {
            if self.short_code_input.len() > 50 {
                self.validation_errors.insert(
                    "short_code".to_string(),
                    "Code too long (max 50 chars)".to_string(),
                );
            }
            // Check for invalid characters
            if !self
                .short_code_input
                .chars()
                .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
            {
                self.validation_errors.insert(
                    "short_code".to_string(),
                    "Only alphanumeric, dash and underscore allowed".to_string(),
                );
            }
        }

        // Validate URL
        if !self.target_url_input.is_empty() {
            if !self.target_url_input.starts_with("http://")
                && !self.target_url_input.starts_with("https://")
            {
                self.validation_errors.insert(
                    "target_url".to_string(),
                    "URL must start with http:// or https://".to_string(),
                );
            }
        } else if matches!(
            self.current_screen,
            CurrentScreen::AddLink | CurrentScreen::EditLink
        ) {
            self.validation_errors
                .insert("target_url".to_string(), "URL is required".to_string());
        }

        // Validate expire time format
        if !self.expire_time_input.is_empty()
            && let Err(e) = TimeParser::parse_expire_time(&self.expire_time_input)
        {
            self.validation_errors
                .insert("expire_time".to_string(), format!("Invalid format: {}", e));
        }
    }

    /// Check if current form has any validation errors
    pub fn has_validation_errors(&self) -> bool {
        !self.validation_errors.is_empty()
    }
}
