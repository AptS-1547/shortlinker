use crate::storages::{ShortLink, Storage, StorageFactory};
use crate::utils::{TimeParser, generate_random_code};
use chrono::Utc;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::sync::Arc;

pub enum CurrentScreen {
    Main,
    AddLink,
    EditLink,
    DeleteConfirm,
    ExportImport,
    Exiting,
}

pub enum CurrentlyEditing {
    ShortCode,
    TargetUrl,
    ExpireTime,
    Password,
}

pub struct App {
    pub storage: Arc<dyn Storage>,
    pub links: HashMap<String, ShortLink>,
    pub current_screen: CurrentScreen,
    pub currently_editing: Option<CurrentlyEditing>,

    // Input fields for add/edit
    pub short_code_input: String,
    pub target_url_input: String,
    pub expire_time_input: String,
    pub password_input: String,
    pub force_overwrite: bool,

    // UI state
    pub selected_index: usize,
    pub status_message: String,
    pub error_message: String,

    // Export/Import
    pub export_path: String,
    pub import_path: String,
}

impl App {
    pub async fn new() -> Result<App, Box<dyn std::error::Error>> {
        let storage = StorageFactory::create().await?;
        let links = storage.load_all().await;

        Ok(App {
            storage,
            links,
            current_screen: CurrentScreen::Main,
            currently_editing: None,
            short_code_input: String::new(),
            target_url_input: String::new(),
            expire_time_input: String::new(),
            password_input: String::new(),
            force_overwrite: false,
            selected_index: 0,
            status_message: String::new(),
            error_message: String::new(),
            export_path: "shortlinks_export.json".to_string(),
            import_path: "shortlinks_import.json".to_string(),
        })
    }

    pub fn refresh_links(&mut self) {
        // This would need to be async, but we'll handle it in the main loop
        // For now, we'll assume links are updated elsewhere
        // TODO: Implement async refresh
    }

    pub fn clear_inputs(&mut self) {
        self.short_code_input.clear();
        self.target_url_input.clear();
        self.expire_time_input.clear();
        self.password_input.clear();
        self.force_overwrite = false;
        self.currently_editing = None;
    }

    pub fn toggle_editing(&mut self) {
        if let Some(edit_mode) = &self.currently_editing {
            match edit_mode {
                CurrentlyEditing::ShortCode => {
                    self.currently_editing = Some(CurrentlyEditing::TargetUrl)
                }
                CurrentlyEditing::TargetUrl => {
                    self.currently_editing = Some(CurrentlyEditing::ExpireTime)
                }
                CurrentlyEditing::ExpireTime => {
                    self.currently_editing = Some(CurrentlyEditing::Password)
                }
                CurrentlyEditing::Password => {
                    self.currently_editing = Some(CurrentlyEditing::ShortCode)
                }
            };
        } else {
            self.currently_editing = Some(CurrentlyEditing::ShortCode);
        }
    }

    pub fn get_selected_link(&self) -> Option<&ShortLink> {
        if self.links.is_empty() {
            return None;
        }
        let keys: Vec<&String> = self.links.keys().collect();
        if self.selected_index < keys.len() {
            self.links.get(keys[self.selected_index])
        } else {
            None
        }
    }

    pub fn move_selection_up(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        }
    }

    pub fn move_selection_down(&mut self) {
        if self.selected_index < self.links.len().saturating_sub(1) {
            self.selected_index += 1;
        }
    }

    pub fn set_status(&mut self, message: String) {
        self.status_message = message;
        self.error_message.clear();
    }

    pub fn set_error(&mut self, message: String) {
        self.error_message = message;
        self.status_message.clear();
    }

    pub async fn save_new_link(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Validate URL format
        if !self.target_url_input.starts_with("http://")
            && !self.target_url_input.starts_with("https://")
        {
            return Err("URL must start with http:// or https://".into());
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
            return Err(format!(
                "Code '{}' already exists. Use force overwrite.",
                final_short_code
            )
            .into());
        }

        let expires_at = if !self.expire_time_input.is_empty() {
            match TimeParser::parse_expire_time(&self.expire_time_input) {
                Ok(dt) => Some(dt),
                Err(e) => return Err(format!("Invalid expiration time: {}", e).into()),
            }
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

        self.storage.set(link).await?;
        self.clear_inputs();
        Ok(())
    }

    pub async fn update_selected_link(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let link = match self.get_selected_link() {
            Some(link) => link,
            None => return Err("No link selected".into()),
        };

        // Validate URL format
        let target_url = if self.target_url_input.is_empty() {
            link.target.clone()
        } else {
            if !self.target_url_input.starts_with("http://")
                && !self.target_url_input.starts_with("https://")
            {
                return Err("URL must start with http:// or https://".into());
            }
            self.target_url_input.clone()
        };

        let expires_at = if !self.expire_time_input.is_empty() {
            match TimeParser::parse_expire_time(&self.expire_time_input) {
                Ok(dt) => Some(dt),
                Err(e) => return Err(format!("Invalid expiration time: {}", e).into()),
            }
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

        self.storage.set(updated_link).await?;
        self.clear_inputs();
        Ok(())
    }

    pub async fn delete_selected_link(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let link = match self.get_selected_link() {
            Some(link) => link,
            None => return Err("No link selected".into()),
        };

        self.storage.remove(&link.code).await?;

        // Adjust selection if necessary
        if self.selected_index >= self.links.len() && self.selected_index > 0 {
            self.selected_index -= 1;
        }

        Ok(())
    }

    pub async fn export_links(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let links_vec: Vec<&ShortLink> = self.links.values().collect();

        let file = File::create(&self.export_path)?;
        let writer = BufWriter::new(file);
        serde_json::to_writer_pretty(writer, &links_vec)?;

        Ok(())
    }

    pub async fn import_links(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let file = File::open(&self.import_path)?;
        let reader = BufReader::new(file);
        let imported_links: Vec<ShortLink> = serde_json::from_reader(reader)?;

        for link in imported_links {
            self.storage.set(link).await?;
        }

        Ok(())
    }
}
