//! Link CRUD operations

use super::state::App;
use crate::errors::ShortlinkerError;
use crate::repository::ShortLink;
use crate::utils::{TimeParser, generate_random_code};
use chrono::Utc;

impl App {
    pub async fn save_new_link(&mut self) -> Result<(), ShortlinkerError> {
        // Validate URL format
        if !self.target_url_input.starts_with("http://")
            && !self.target_url_input.starts_with("https://")
        {
            return Err(ShortlinkerError::validation(
                "URL must start with http:// or https://",
            ));
        }

        let config = crate::system::app_config::get_config();
        let random_code_length = config.features.random_code_length;

        let final_short_code = if self.short_code_input.is_empty() {
            let code = generate_random_code(random_code_length);
            self.set_status(format!("Generated random code: {}", code));
            code
        } else {
            self.short_code_input.clone()
        };

        // Check if short code already exists
        if self.links.contains_key(&final_short_code) && !self.force_overwrite {
            return Err(ShortlinkerError::validation(format!(
                "Code '{}' already exists. Use force overwrite.",
                final_short_code
            )));
        }

        let expires_at = if !self.expire_time_input.is_empty() {
            Some(
                TimeParser::parse_expire_time(&self.expire_time_input)
                    .map_err(ShortlinkerError::date_parse)?,
            )
        } else {
            None
        };

        let link = ShortLink {
            code: final_short_code.clone(),
            target: self.target_url_input.clone(),
            created_at: Utc::now(),
            expires_at,
            password: if self.password_input.is_empty() {
                None
            } else {
                Some(self.password_input.clone())
            },
            click: 0,
        };

        self.repository.set(link).await?;
        self.clear_inputs();
        Ok(())
    }

    pub async fn update_selected_link(&mut self) -> Result<(), ShortlinkerError> {
        let link = match self.get_selected_link() {
            Some(link) => link,
            None => return Err(ShortlinkerError::validation("No link selected")),
        };

        // Validate URL format
        let target_url = if self.target_url_input.is_empty() {
            link.target.clone()
        } else {
            if !self.target_url_input.starts_with("http://")
                && !self.target_url_input.starts_with("https://")
            {
                return Err(ShortlinkerError::validation(
                    "URL must start with http:// or https://",
                ));
            }
            self.target_url_input.clone()
        };

        let expires_at = if !self.expire_time_input.is_empty() {
            Some(
                TimeParser::parse_expire_time(&self.expire_time_input)
                    .map_err(ShortlinkerError::date_parse)?,
            )
        } else {
            link.expires_at
        };

        let password = if self.password_input.is_empty() {
            link.password.clone()
        } else {
            Some(self.password_input.clone())
        };

        let updated_link = ShortLink {
            code: link.code.clone(),
            target: target_url,
            created_at: link.created_at,
            expires_at,
            password,
            click: link.click,
        };

        self.repository.set(updated_link).await?;
        self.clear_inputs();
        Ok(())
    }

    pub async fn delete_selected_link(&mut self) -> Result<(), ShortlinkerError> {
        let link = match self.get_selected_link() {
            Some(link) => link,
            None => return Err(ShortlinkerError::validation("No link selected")),
        };

        self.repository.remove(&link.code).await?;

        // Adjust selection if necessary
        if self.selected_index >= self.links.len() && self.selected_index > 0 {
            self.selected_index -= 1;
        }

        Ok(())
    }
}
