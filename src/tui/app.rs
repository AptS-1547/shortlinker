use crate::errors::ShortlinkerError;
use crate::repository::{Repository, RepositoryFactory, ShortLink};
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
    Search,
    Help,
    ViewDetails,
    FileBrowser,    // 文件浏览器
    ExportFileName, // 输入导出文件名
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
    pub filtered_links: Vec<String>, // List of codes matching search
    pub is_searching: bool,

    // UI state
    pub selected_index: usize,
    pub status_message: String,
    pub error_message: String,

    // Export/Import
    pub export_path: String,
    pub import_path: String,

    // File browser state
    pub current_dir: std::path::PathBuf,      // 当前浏览目录
    pub dir_entries: Vec<std::path::PathBuf>, // 目录中的文件/文件夹列表
    pub browser_selected_index: usize,        // 文件浏览器选中索引
    pub export_filename_input: String,        // 导出文件名输入
}

impl App {
    pub async fn new() -> Result<App, ShortlinkerError> {
        let repository = RepositoryFactory::create().await?;
        let links = repository.load_all().await;

        // 初始化当前目录为用户主目录或当前工作目录
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
        // Refresh links from repository
        self.links = self.repository.load_all().await;
        // Notify server to reload
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

    pub fn jump_to_top(&mut self) {
        self.selected_index = 0;
    }

    pub fn jump_to_bottom(&mut self) {
        if !self.links.is_empty() {
            self.selected_index = self.links.len() - 1;
        }
    }

    pub fn page_up(&mut self) {
        if self.selected_index >= 10 {
            self.selected_index -= 10;
        } else {
            self.selected_index = 0;
        }
    }

    pub fn page_down(&mut self) {
        let max_index = self.links.len().saturating_sub(1);
        if self.selected_index + 10 <= max_index {
            self.selected_index += 10;
        } else {
            self.selected_index = max_index;
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

    pub async fn export_links(&mut self) -> Result<(), ShortlinkerError> {
        let links_vec: Vec<&ShortLink> = self.links.values().collect();

        let file = File::create(&self.export_path)
            .map_err(|e| ShortlinkerError::file_operation(e.to_string()))?;
        let writer = BufWriter::new(file);
        serde_json::to_writer_pretty(writer, &links_vec)
            .map_err(|e| ShortlinkerError::serialization(e.to_string()))?;

        Ok(())
    }

    pub async fn import_links(&mut self) -> Result<(), ShortlinkerError> {
        let file = File::open(&self.import_path)
            .map_err(|e| ShortlinkerError::file_operation(e.to_string()))?;
        let reader = BufReader::new(file);
        let imported_links: Vec<ShortLink> = serde_json::from_reader(reader)
            .map_err(|e| ShortlinkerError::serialization(e.to_string()))?;

        for link in imported_links {
            self.repository.set(link).await?;
        }

        Ok(())
    }

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

    /// Filter links based on search query
    pub fn filter_links(&mut self) {
        if self.search_input.is_empty() {
            self.filtered_links.clear();
            self.is_searching = false;
            return;
        }

        self.is_searching = true;
        let query = self.search_input.to_lowercase();

        self.filtered_links = self
            .links
            .iter()
            .filter(|(code, link)| {
                code.to_lowercase().contains(&query) || link.target.to_lowercase().contains(&query)
            })
            .map(|(code, _)| code.clone())
            .collect();

        // Reset selection to first item
        self.selected_index = 0;
    }

    /// Get links to display (filtered or all)
    pub fn get_display_links(&self) -> Vec<(&String, &ShortLink)> {
        if self.is_searching && !self.filtered_links.is_empty() {
            self.filtered_links
                .iter()
                .filter_map(|code| self.links.get(code).map(|link| (code, link)))
                .collect()
        } else if self.is_searching {
            // Searching but no results
            Vec::new()
        } else {
            // Not searching, show all
            self.links.iter().collect()
        }
    }

    // ========== File Browser Methods ==========

    /// Load directory entries for file browser
    pub fn load_directory(&mut self) -> Result<(), ShortlinkerError> {
        use std::fs;

        self.dir_entries.clear();
        self.browser_selected_index = 0;

        // Add parent directory entry if not at root
        if self.current_dir.parent().is_some() {
            self.dir_entries.push(self.current_dir.join(".."));
        }

        // Read directory entries
        let entries = fs::read_dir(&self.current_dir).map_err(|e| {
            ShortlinkerError::file_operation(format!("Failed to read directory: {}", e))
        })?;

        // Sort entries: directories first, then files
        let mut dirs = Vec::new();
        let mut files = Vec::new();

        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                dirs.push(path);
            } else if path.extension().and_then(|s| s.to_str()) == Some("json") {
                // Only show JSON files for import
                files.push(path);
            }
        }

        dirs.sort();
        files.sort();

        self.dir_entries.extend(dirs);
        self.dir_entries.extend(files);

        Ok(())
    }

    /// Navigate into selected directory or select file
    pub fn browser_navigate(&mut self) -> Result<Option<std::path::PathBuf>, ShortlinkerError> {
        if self.dir_entries.is_empty() {
            return Ok(None);
        }

        let selected = &self.dir_entries[self.browser_selected_index];

        if selected.is_dir() {
            // Navigate into directory
            self.current_dir = selected
                .canonicalize()
                .map_err(|e| ShortlinkerError::file_operation(e.to_string()))?;
            self.load_directory()?;
            Ok(None)
        } else {
            // File selected
            Ok(Some(selected.clone()))
        }
    }

    /// Move selection up in file browser
    pub fn browser_move_up(&mut self) {
        if self.browser_selected_index > 0 {
            self.browser_selected_index -= 1;
        }
    }

    /// Move selection down in file browser
    pub fn browser_move_down(&mut self) {
        if !self.dir_entries.is_empty() && self.browser_selected_index < self.dir_entries.len() - 1
        {
            self.browser_selected_index += 1;
        }
    }

    /// Get selected entry in browser
    pub fn get_selected_entry(&self) -> Option<&std::path::PathBuf> {
        self.dir_entries.get(self.browser_selected_index)
    }
}
