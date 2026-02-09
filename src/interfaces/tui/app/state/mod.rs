//! App state definition and basic state management
//!
//! 包含核心 App 结构和基础状态管理，以及拆分后的子状态模块

mod form_state;

pub use form_state::{EditingField, FormState};

use crate::errors::ShortlinkerError;
use crate::storage::{SeaOrmStorage, ShortLink, StorageFactory};

use crate::metrics_core::NoopMetrics;

use ratatui::widgets::TableState;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

/// 排序列
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortColumn {
    Code,
    Url,
    Clicks,
    Status,
}

impl SortColumn {
    /// 循环切换到下一个排序列
    pub fn next(self) -> Self {
        match self {
            Self::Code => Self::Url,
            Self::Url => Self::Clicks,
            Self::Clicks => Self::Status,
            Self::Status => Self::Code,
        }
    }
}

/// 当前屏幕
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CurrentScreen {
    Main,
    AddLink,
    EditLink,
    DeleteConfirm,
    BatchDeleteConfirm,
    ExportImport,
    Exiting,
    Search,
    Help,
    ViewDetails,
    FileBrowser,
    ExportFileName,
}

/// 当前编辑的字段（保留以兼容现有代码，可通过 From trait 与 EditingField 互转）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CurrentlyEditing {
    ShortCode,
    TargetUrl,
    ExpireTime,
    Password,
}

impl From<EditingField> for CurrentlyEditing {
    fn from(field: EditingField) -> Self {
        match field {
            EditingField::ShortCode => Self::ShortCode,
            EditingField::TargetUrl => Self::TargetUrl,
            EditingField::ExpireTime => Self::ExpireTime,
            EditingField::Password => Self::Password,
        }
    }
}

impl From<CurrentlyEditing> for EditingField {
    fn from(field: CurrentlyEditing) -> Self {
        match field {
            CurrentlyEditing::ShortCode => Self::ShortCode,
            CurrentlyEditing::TargetUrl => Self::TargetUrl,
            CurrentlyEditing::ExpireTime => Self::ExpireTime,
            CurrentlyEditing::Password => Self::Password,
        }
    }
}

pub struct App {
    pub storage: Arc<SeaOrmStorage>,
    pub links: HashMap<String, ShortLink>,
    pub current_screen: CurrentScreen,

    // Form state for add/edit
    pub form: FormState,

    // Search functionality
    pub search_input: String,
    pub filtered_links: Vec<String>,
    pub is_searching: bool,
    pub inline_search_mode: bool,

    // Sorting
    pub sort_column: Option<SortColumn>,
    pub sort_ascending: bool,

    // Batch selection
    pub selected_items: HashSet<String>,

    // UI state
    pub selected_index: usize,
    pub table_state: TableState,
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
        let storage = StorageFactory::create(NoopMetrics::arc()).await?;
        let links = storage.load_all().await?;

        let current_dir = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));

        let mut table_state = TableState::default();
        table_state.select(Some(0));

        Ok(App {
            storage,
            links,
            current_screen: CurrentScreen::Main,
            form: FormState::new(),
            search_input: String::new(),
            filtered_links: Vec::new(),
            is_searching: false,
            inline_search_mode: false,
            sort_column: None,
            sort_ascending: true,
            selected_items: HashSet::new(),
            selected_index: 0,
            table_state,
            status_message: String::new(),
            error_message: String::new(),
            export_path: "shortlinks_export.csv".to_string(),
            import_path: "shortlinks_import.csv".to_string(),
            current_dir,
            dir_entries: Vec::new(),
            browser_selected_index: 0,
            export_filename_input: String::new(),
        })
    }

    pub async fn refresh_links(&mut self) -> Result<(), ShortlinkerError> {
        self.links = self.storage.load_all().await?;
        // Notify server via IPC to reload (best effort, don't fail if server not running)
        use crate::system::ipc::{self, IpcResponse};
        use crate::system::reload::ReloadTarget;
        if ipc::is_server_running() {
            match ipc::reload(ReloadTarget::Data).await {
                Ok(IpcResponse::ReloadResult {
                    success: false,
                    message,
                    ..
                }) => {
                    tracing::warn!("Server reload failed: {:?}", message);
                }
                Err(e) => {
                    tracing::warn!("Failed to notify server via IPC: {}", e);
                }
                _ => {} // Success or other responses
            }
        }
        Ok(())
    }

    pub fn clear_inputs(&mut self) {
        self.form.clear();
    }

    pub fn toggle_editing(&mut self) {
        self.form.toggle_field();
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

    /// Get links to display (filtered or all, with sorting applied)
    pub fn get_display_links(&self) -> Vec<(&String, &ShortLink)> {
        let mut links: Vec<(&String, &ShortLink)> =
            if self.is_searching && !self.filtered_links.is_empty() {
                self.filtered_links
                    .iter()
                    .filter_map(|code| self.links.get(code).map(|link| (code, link)))
                    .collect()
            } else if self.is_searching {
                Vec::new()
            } else {
                self.links.iter().collect()
            };

        // Apply sorting if a sort column is set
        if let Some(column) = self.sort_column {
            links.sort_by(|a, b| {
                let cmp = match column {
                    SortColumn::Code => a.0.cmp(b.0),
                    SortColumn::Url => a.1.target.cmp(&b.1.target),
                    SortColumn::Clicks => a.1.click.cmp(&b.1.click),
                    SortColumn::Status => {
                        let status_a = !a.1.is_expired();
                        let status_b = !b.1.is_expired();
                        status_a.cmp(&status_b)
                    }
                };
                if self.sort_ascending {
                    cmp
                } else {
                    cmp.reverse()
                }
            });
        }

        links
    }

    /// Cycle through sort columns
    pub fn cycle_sort_column(&mut self) {
        self.sort_column = Some(match self.sort_column {
            None => SortColumn::Code,
            Some(col) => col.next(),
        });
    }

    /// Toggle sort direction
    pub fn toggle_sort_direction(&mut self) {
        self.sort_ascending = !self.sort_ascending;
    }

    /// Toggle selection of current item
    pub fn toggle_selection(&mut self) {
        if let Some(link) = self.get_selected_link() {
            let code = link.code.clone();
            if self.selected_items.contains(&code) {
                self.selected_items.remove(&code);
            } else {
                self.selected_items.insert(code);
            }
        }
    }

    /// Select all visible links
    #[allow(dead_code)]
    pub fn select_all(&mut self) {
        let codes: Vec<String> = self
            .get_display_links()
            .iter()
            .map(|(k, _)| (*k).clone())
            .collect();
        self.selected_items.extend(codes);
    }

    /// Clear all selections
    pub fn clear_selection(&mut self) {
        self.selected_items.clear();
    }

    /// Check if an item is selected
    pub fn is_selected(&self, code: &str) -> bool {
        self.selected_items.contains(code)
    }
}
