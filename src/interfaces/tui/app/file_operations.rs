//! File operations: export, import, and file browser

use super::state::App;
use crate::errors::ShortlinkerError;
use crate::services::{ImportBatchResult, ImportLinkItemRich};
use crate::storage::ShortLink;
use crate::utils::csv_handler;

impl App {
    pub async fn export_links(&mut self) -> Result<(), ShortlinkerError> {
        let links = self.link_client.export_links().await?;
        let links_ref: Vec<&ShortLink> = links.iter().collect();

        // Use csv_handler to export
        csv_handler::export_to_csv(&links_ref, &self.export_path)?;

        Ok(())
    }

    pub async fn import_links(
        &mut self,
        overwrite: bool,
    ) -> Result<ImportBatchResult, ShortlinkerError> {
        let imported_links: Vec<ShortLink> = csv_handler::import_from_csv(&self.import_path)?;

        let items: Vec<ImportLinkItemRich> = imported_links
            .into_iter()
            .map(|link| ImportLinkItemRich {
                code: link.code,
                target: link.target,
                created_at: link.created_at,
                expires_at: link.expires_at,
                password: link.password,
                click_count: link.click,
            })
            .collect();

        let result = self.link_client.import_links(items, overwrite).await?;

        Ok(result)
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
            } else if let Some(ext) = path.extension().and_then(|s| s.to_str())
                && ext.eq_ignore_ascii_case("csv")
            {
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
