//! App state definition and basic state management

use crate::errors::ShortlinkerError;
use crate::repository::{Repository, RepositoryFactory, ShortLink};

use std::collections::HashMap;
use std::sync::Arc;

pub enum CurrentScreen {
    Main,
    AddLink,
    EditLink,
    DeleteConfirm,
    ExportImport,
    Exiting,
    Search,
    Help,
    ViewDetails,
    FileBrowser,
    ExportFileName,
}

pub enum CurrentlyEditing {
    ShortCode,
    TargetUrl,
    ExpireTime,
    Password,
}

pub struct App {
    pub repository: Arc<dyn Repository>,
    pub links: HashMap<String, ShortLink>,
    pub current_screen: CurrentScreen,
    pub currently_editing: Option<CurrentlyEditing>,

    // Input fields for add/edit
    pub short_code_input: String,
    pub target_url_input: String,
    pub expire_time_input: String,
    pub password_input: String,
    pub force_overwrite: bool,

    // Validation errors for real-time feedback
    pub validation_errors: HashMap<String, String>,

    // Search functionality
    pub search_input: String,
    pub filtered_links: Vec<String>,
    pub is_searching: bool,

    // UI state
    pub selected_index: usize,
    pub status_message: String,
    pub error_message: String,

    // Export/Import
    pub export_path: String,
    pub import_path: String,

    // File browser state
    pub current_dir: std::path::PathBuf,
    pub dir_entries: Vec<std::path::PathBuf>,
    pub browser_selected_index: usize,
    pub export_filename_input: String,
}

impl App {
    pub async fn new() -> Result<App, ShortlinkerError> {
        let repository = RepositoryFactory::create().await?;
        let links = repository.load_all().await;

        let current_dir = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));

        Ok(App {
            repository,
            links,
            current_screen: CurrentScreen::Main,
            currently_editing: None,
            short_code_input: String::new(),
            target_url_input: String::new(),
            expire_time_input: String::new(),
            password_input: String::new(),
            force_overwrite: false,
            validation_errors: HashMap::new(),
            search_input: String::new(),
            filtered_links: Vec::new(),
            is_searching: false,
            selected_index: 0,
            status_message: String::new(),
            error_message: String::new(),
            export_path: "shortlinks_export.json".to_string(),
            import_path: "shortlinks_import.json".to_string(),
            current_dir,
            dir_entries: Vec::new(),
            browser_selected_index: 0,
            export_filename_input: String::new(),
        })
    }

    pub async fn refresh_links(&mut self) -> Result<(), ShortlinkerError> {
        self.links = self.repository.load_all().await;
        if let Err(e) = crate::system::platform::notify_server() {
            return Err(ShortlinkerError::notify_server(format!(
                "Failed to notify server: {}",
                e
            )));
        }
        Ok(())
    }

    pub fn clear_inputs(&mut self) {
        self.short_code_input.clear();
        self.target_url_input.clear();
        self.expire_time_input.clear();
        self.password_input.clear();
        self.force_overwrite = false;
        self.validation_errors.clear();
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

    pub fn set_status(&mut self, message: String) {
        self.status_message = message;
        self.error_message.clear();
    }

    pub fn set_error(&mut self, message: String) {
        self.error_message = message;
        self.status_message.clear();
    }

    /// Get links to display (filtered or all)
    pub fn get_display_links(&self) -> Vec<(&String, &ShortLink)> {
        if self.is_searching && !self.filtered_links.is_empty() {
            self.filtered_links
                .iter()
                .filter_map(|code| self.links.get(code).map(|link| (code, link)))
                .collect()
        } else if self.is_searching {
            Vec::new()
        } else {
            self.links.iter().collect()
        }
    }
}
