//! File operations: export, import, and file browser

use super::state::App;
use crate::errors::ShortlinkerError;
use crate::storage::ShortLink;
use crate::utils::password::process_new_password;
use crate::utils::url_validator::validate_url;
use std::fs::File;
use std::io::{BufReader, BufWriter};

impl App {
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
        let mut imported_links: Vec<ShortLink> = serde_json::from_reader(reader)
            .map_err(|e| ShortlinkerError::serialization(e.to_string()))?;

        for link in &mut imported_links {
            // Validate URL
            validate_url(&link.target).map_err(|e| {
                ShortlinkerError::validation(format!("Invalid URL for code '{}': {}", link.code, e))
            })?;

            // Process password (hash if plaintext, keep if already hashed)
            link.password = process_new_password(link.password.as_deref()).map_err(|e| {
                ShortlinkerError::validation(format!(
                    "Failed to process password for code '{}': {}",
                    link.code, e
                ))
            })?;
        }

        for link in imported_links {
            self.storage.set(link).await?;
        }

        Ok(())
    }

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
    #[allow(dead_code)]
    pub fn get_selected_entry(&self) -> Option<&std::path::PathBuf> {
        self.dir_entries.get(self.browser_selected_index)
    }
}
