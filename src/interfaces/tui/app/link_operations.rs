//! Link CRUD operations

use super::state::App;
use crate::errors::ShortlinkerError;

impl App {
    pub async fn save_new_link(&mut self) -> Result<(), ShortlinkerError> {
        // Check for validation errors first
        self.validate_inputs();
        if self.has_validation_errors() {
            return Err(ShortlinkerError::validation(
                "Please fix validation errors before saving",
            ));
        }

        let code = if self.form.short_code.is_empty() {
            None
        } else {
            Some(self.form.short_code.clone())
        };

        let expires_at = if self.form.expire_time.is_empty() {
            None
        } else {
            Some(self.form.expire_time.clone())
        };

        let password = if self.form.password.is_empty() {
            None
        } else {
            Some(self.form.password.clone())
        };

        let result = self
            .link_client
            .create_link(
                code,
                self.form.target_url.clone(),
                self.form.force_overwrite,
                expires_at,
                password,
            )
            .await?;

        if result.generated_code {
            self.set_status(format!("Generated random code: {}", result.link.code));
        }

        self.clear_inputs();
        Ok(())
    }

    pub async fn update_selected_link(&mut self) -> Result<(), ShortlinkerError> {
        let link = match self.get_selected_link() {
            Some(link) => link,
            None => return Err(ShortlinkerError::validation("No link selected")),
        };

        let code = link.code.clone();

        let target = if self.form.target_url.is_empty() {
            link.target.clone()
        } else {
            self.form.target_url.clone()
        };

        let expires_at = if self.form.expire_time.is_empty() {
            None
        } else {
            Some(self.form.expire_time.clone())
        };

        let password = if self.form.password.is_empty() {
            None
        } else {
            Some(self.form.password.clone())
        };

        self.link_client
            .update_link(code, target, expires_at, password)
            .await?;

        self.clear_inputs();
        Ok(())
    }

    pub async fn delete_selected_link(&mut self) -> Result<(), ShortlinkerError> {
        let link = match self.get_selected_link() {
            Some(link) => link,
            None => return Err(ShortlinkerError::validation("No link selected")),
        };

        let code = link.code.clone();
        self.link_client.delete_link(code).await?;

        // Adjust selection if necessary
        if self.selected_index >= self.links.len() && self.selected_index > 0 {
            self.selected_index -= 1;
        }

        Ok(())
    }
}
