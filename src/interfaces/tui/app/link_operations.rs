//! Link CRUD operations

use super::state::App;
use crate::errors::ShortlinkerError;
use crate::storage::ShortLink;
use crate::utils::password::{process_new_password, process_update_password};
use crate::utils::url_validator::validate_url;
use crate::utils::{TimeParser, generate_random_code};
use chrono::Utc;

impl App {
    pub async fn save_new_link(&mut self) -> Result<(), ShortlinkerError> {
        // Check for validation errors first
        self.validate_inputs();
        if self.has_validation_errors() {
            return Err(ShortlinkerError::validation(
                "Please fix validation errors before saving",
            ));
        }

        // Validate URL format
        validate_url(&self.form.target_url)
            .map_err(|e| ShortlinkerError::validation(e.to_string()))?;

        let rt = crate::config::get_runtime_config();
        let random_code_length =
            rt.get_usize_or(crate::config::keys::FEATURES_RANDOM_CODE_LENGTH, 6);

        let final_short_code = if self.form.short_code.is_empty() {
            let code = generate_random_code(random_code_length);
            self.set_status(format!("Generated random code: {}", code));
            code
        } else {
            self.form.short_code.clone()
        };

        // Check if short code already exists
        if self.links.contains_key(&final_short_code) && !self.form.force_overwrite {
            return Err(ShortlinkerError::validation(format!(
                "Code '{}' already exists. Use force overwrite.",
                final_short_code
            )));
        }

        let expires_at = if !self.form.expire_time.is_empty() {
            Some(
                TimeParser::parse_expire_time(&self.form.expire_time)
                    .map_err(ShortlinkerError::link_invalid_expire_time)?,
            )
        } else {
            None
        };

        // Process password (hash if needed)
        let password = process_new_password(if self.form.password.is_empty() {
            None
        } else {
            Some(&self.form.password)
        })
        .map_err(|e| ShortlinkerError::validation(e.to_string()))?;

        let link = ShortLink {
            code: final_short_code.clone(),
            target: self.form.target_url.clone(),
            created_at: Utc::now(),
            expires_at,
            password,
            click: 0,
        };

        self.storage.set(link).await?;
        self.clear_inputs();
        Ok(())
    }

    pub async fn update_selected_link(&mut self) -> Result<(), ShortlinkerError> {
        let link = match self.get_selected_link() {
            Some(link) => link,
            None => return Err(ShortlinkerError::validation("No link selected")),
        };

        // Validate URL format
        let target_url = if self.form.target_url.is_empty() {
            link.target.clone()
        } else {
            validate_url(&self.form.target_url)
                .map_err(|e| ShortlinkerError::validation(e.to_string()))?;
            self.form.target_url.clone()
        };

        let expires_at = if !self.form.expire_time.is_empty() {
            Some(
                TimeParser::parse_expire_time(&self.form.expire_time)
                    .map_err(ShortlinkerError::link_invalid_expire_time)?,
            )
        } else {
            link.expires_at
        };

        // Process password (hash if needed)
        let password = process_update_password(
            if self.form.password.is_empty() {
                None
            } else {
                Some(&self.form.password)
            },
            link.password.clone(),
        )
        .map_err(|e| ShortlinkerError::validation(e.to_string()))?;

        let updated_link = ShortLink {
            code: link.code.clone(),
            target: target_url,
            created_at: link.created_at,
            expires_at,
            password,
            click: link.click,
        };

        self.storage.set(updated_link).await?;
        self.clear_inputs();
        Ok(())
    }

    pub async fn delete_selected_link(&mut self) -> Result<(), ShortlinkerError> {
        let link = match self.get_selected_link() {
            Some(link) => link,
            None => return Err(ShortlinkerError::validation("No link selected")),
        };

        self.storage.remove(&link.code).await?;

        // Adjust selection if necessary
        if self.selected_index >= self.links.len() && self.selected_index > 0 {
            self.selected_index -= 1;
        }

        Ok(())
    }
}
