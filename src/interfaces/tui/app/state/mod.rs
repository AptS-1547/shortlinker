//! App state definition and basic state management
//!
//! 包含核心 App 结构和基础状态管理，以及拆分后的子状态模块

mod form_state;
mod system_state;

pub use form_state::{EditingField, FormState};
pub use system_state::{ConfigListItem, PasswordField, SystemState};

use crate::client::{ConfigClient, LinkClient, ServiceContext, SystemClient};
use crate::errors::ShortlinkerError;
use crate::storage::{SeaOrmStorage, ShortLink, StorageFactory};

use crate::metrics_core::NoopMetrics;

use ratatui::widgets::TableState;
use std::collections::HashSet;
use std::sync::Arc;

/// TUI 每页加载的链接数量
pub const TUI_PAGE_SIZE: u64 = 100;

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
    // System operations
    SystemMenu,
    ServerStatus,
    ConfigList,
    ConfigEdit,
    ConfigResetConfirm,
    PasswordReset,
    ImportModeSelect,
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
    pub link_client: LinkClient,
    pub config_client: ConfigClient,
    pub system_client: SystemClient,
    pub current_screen: CurrentScreen,

    // 分页数据
    /// 当前页的链接列表（可能经过页内排序）
    pub page_links: Vec<ShortLink>,
    /// 当前页码（1-based）
    pub current_page: u64,
    /// 每页大小
    pub page_size: u64,
    /// 数据库中匹配当前过滤条件的总链接数
    pub total_count: u64,

    // Form state for add/edit
    pub form: FormState,

    // System operations state
    pub system: SystemState,

    // Search functionality
    pub search_input: String,
    /// 已提交到数据库的搜索词（Enter 后生效）
    pub search_query: Option<String>,
    pub is_searching: bool,
    pub inline_search_mode: bool,

    // Sorting (页内排序)
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

    // Virtual rendering
    /// 虚拟滚动偏移量（页内）
    pub scroll_offset: usize,
    /// 上次渲染时的可见行数（由 draw_main_screen 回写）
    pub last_visible_height: usize,
}

impl App {
    pub async fn new() -> Result<App, ShortlinkerError> {
        let storage = StorageFactory::create(NoopMetrics::arc()).await?;

        let ctx = Arc::new(ServiceContext::with_storage(storage.clone()));
        let link_client = LinkClient::new(ctx.clone());
        let config_client = ConfigClient::new(ctx);
        let system_client = SystemClient::new();

        // 只加载第一页数据，而不是全量
        let (page_links, total_count) = link_client
            .list_links(1, TUI_PAGE_SIZE, None)
            .await
            .map_err(|e| {
                ShortlinkerError::database_operation(format!("Failed to load links: {}", e))
            })?;

        let current_dir = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));

        let mut table_state = TableState::default();
        table_state.select(Some(0));

        Ok(App {
            storage,
            link_client,
            config_client,
            system_client,
            current_screen: CurrentScreen::Main,
            page_links,
            current_page: 1,
            page_size: TUI_PAGE_SIZE,
            total_count,
            form: FormState::new(),
            system: SystemState::default(),
            search_input: String::new(),
            search_query: None,
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
            scroll_offset: 0,
            last_visible_height: 20,
        })
    }

    /// 加载指定页数据
    pub async fn load_page(&mut self, page: u64) -> Result<(), ShortlinkerError> {
        let (links, total) = self
            .link_client
            .list_links(page, self.page_size, self.search_query.clone())
            .await
            .map_err(|e| {
                ShortlinkerError::database_operation(format!("Failed to load page: {}", e))
            })?;

        self.page_links = links;
        self.total_count = total;
        self.current_page = page;
        self.apply_sort();
        Ok(())
    }

    /// 刷新当前页数据，并通知服务器重新加载
    pub async fn refresh_links(&mut self) -> Result<(), ShortlinkerError> {
        self.load_page(self.current_page).await?;
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

    /// 执行搜索：提交搜索词到数据库，加载第一页结果
    pub async fn execute_search(&mut self) -> Result<(), ShortlinkerError> {
        if self.search_input.is_empty() {
            self.search_query = None;
            self.is_searching = false;
        } else {
            self.search_query = Some(self.search_input.clone());
            self.is_searching = true;
        }
        self.selected_index = 0;
        self.scroll_offset = 0;
        self.load_page(1).await
    }

    /// 清除搜索，恢复全量显示
    pub async fn clear_search(&mut self) -> Result<(), ShortlinkerError> {
        self.search_input.clear();
        self.search_query = None;
        self.is_searching = false;
        self.inline_search_mode = false;
        self.selected_index = 0;
        self.scroll_offset = 0;
        self.load_page(1).await
    }

    pub fn clear_inputs(&mut self) {
        self.form.clear();
    }

    pub fn toggle_editing(&mut self) {
        self.form.toggle_field();
    }

    pub fn get_selected_link(&self) -> Option<&ShortLink> {
        self.page_links.get(self.selected_index)
    }

    pub fn set_status(&mut self, message: String) {
        self.status_message = message;
        self.error_message.clear();
    }

    pub fn set_error(&mut self, message: String) {
        self.error_message = message;
        self.status_message.clear();
    }

    /// 当前页的链接数量
    pub fn display_count(&self) -> usize {
        self.page_links.len()
    }

    /// 总页数
    pub fn total_pages(&self) -> u64 {
        if self.page_size == 0 {
            return 1;
        }
        self.total_count.div_ceil(self.page_size).max(1)
    }

    /// 是否有下一页
    pub fn has_next_page(&self) -> bool {
        self.current_page < self.total_pages()
    }

    /// 是否有上一页
    pub fn has_prev_page(&self) -> bool {
        self.current_page > 1
    }

    /// 对当前页数据应用页内排序
    fn apply_sort(&mut self) {
        if let Some(column) = self.sort_column {
            let ascending = self.sort_ascending;
            self.page_links.sort_by(|a, b| {
                let cmp = match column {
                    SortColumn::Code => a.code.cmp(&b.code),
                    SortColumn::Url => a.target.cmp(&b.target),
                    SortColumn::Clicks => a.click.cmp(&b.click),
                    SortColumn::Status => {
                        let status_a = !a.is_expired();
                        let status_b = !b.is_expired();
                        status_a.cmp(&status_b)
                    }
                };
                if ascending { cmp } else { cmp.reverse() }
            });
        }
    }

    /// Cycle through sort columns
    pub fn cycle_sort_column(&mut self) {
        self.sort_column = Some(match self.sort_column {
            None => SortColumn::Code,
            Some(col) => col.next(),
        });
        self.apply_sort();
    }

    /// Toggle sort direction
    pub fn toggle_sort_direction(&mut self) {
        self.sort_ascending = !self.sort_ascending;
        self.apply_sort();
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
        let codes: Vec<String> = self.page_links.iter().map(|l| l.code.clone()).collect();
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

    /// 确保 selected_index 不越界，并同步 scroll_offset
    pub fn clamp_selection(&mut self) {
        if self.page_links.is_empty() {
            self.selected_index = 0;
        } else if self.selected_index >= self.page_links.len() {
            self.selected_index = self.page_links.len() - 1;
        }
        self.adjust_scroll_offset();
        self.table_state.select(Some(self.selected_index));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use std::sync::Once;

    static INIT: Once = Once::new();

    fn init_test_env() {
        INIT.call_once(|| {
            crate::config::init_config();
        });
    }

    fn make_link(code: &str, target: &str, clicks: usize) -> ShortLink {
        ShortLink {
            code: code.to_string(),
            target: target.to_string(),
            created_at: Utc::now(),
            expires_at: None,
            password: None,
            click: clicks,
        }
    }

    fn make_links(n: usize) -> Vec<ShortLink> {
        (0..n)
            .map(|i| {
                make_link(
                    &format!("code{:03}", i),
                    &format!("https://example.com/{}", i),
                    i,
                )
            })
            .collect()
    }

    async fn test_app(
        page_links: Vec<ShortLink>,
        total_count: u64,
        page_size: u64,
        current_page: u64,
    ) -> App {
        init_test_env();

        let storage = crate::storage::SeaOrmStorage::new(
            "sqlite::memory:",
            "sqlite",
            crate::metrics_core::NoopMetrics::arc(),
        )
        .await
        .expect("Failed to create in-memory storage");
        let storage = Arc::new(storage);

        let ctx = Arc::new(ServiceContext::with_storage(storage.clone()));
        let link_client = LinkClient::new(ctx.clone());
        let config_client = ConfigClient::new(ctx);
        let system_client = SystemClient::new();

        let mut table_state = TableState::default();
        table_state.select(Some(0));

        App {
            storage,
            link_client,
            config_client,
            system_client,
            current_screen: CurrentScreen::Main,
            page_links,
            current_page,
            page_size,
            total_count,
            form: FormState::new(),
            system: SystemState::default(),
            search_input: String::new(),
            search_query: None,
            is_searching: false,
            inline_search_mode: false,
            sort_column: None,
            sort_ascending: true,
            selected_items: HashSet::new(),
            selected_index: 0,
            table_state,
            status_message: String::new(),
            error_message: String::new(),
            export_path: String::new(),
            import_path: String::new(),
            current_dir: std::path::PathBuf::from("."),
            dir_entries: Vec::new(),
            browser_selected_index: 0,
            export_filename_input: String::new(),
            scroll_offset: 0,
            last_visible_height: 20,
        }
    }

    // ── total_pages ──

    #[tokio::test]
    async fn total_pages_zero_items() {
        let app = test_app(vec![], 0, 100, 1).await;
        assert_eq!(app.total_pages(), 1);
    }

    #[tokio::test]
    async fn total_pages_one_item() {
        let app = test_app(make_links(1), 1, 100, 1).await;
        assert_eq!(app.total_pages(), 1);
    }

    #[tokio::test]
    async fn total_pages_exact_division() {
        let app = test_app(vec![], 200, 100, 1).await;
        assert_eq!(app.total_pages(), 2);
    }

    #[tokio::test]
    async fn total_pages_with_remainder() {
        let app = test_app(vec![], 201, 100, 1).await;
        assert_eq!(app.total_pages(), 3);
    }

    #[tokio::test]
    async fn total_pages_zero_page_size() {
        let app = test_app(vec![], 100, 0, 1).await;
        assert_eq!(app.total_pages(), 1);
    }

    // ── has_next_page / has_prev_page ──

    #[tokio::test]
    async fn first_page_of_many() {
        let app = test_app(vec![], 200, 100, 1).await;
        assert!(app.has_next_page());
        assert!(!app.has_prev_page());
    }

    #[tokio::test]
    async fn last_page_of_many() {
        let app = test_app(vec![], 200, 100, 2).await;
        assert!(!app.has_next_page());
        assert!(app.has_prev_page());
    }

    #[tokio::test]
    async fn single_page() {
        let app = test_app(make_links(5), 5, 100, 1).await;
        assert!(!app.has_next_page());
        assert!(!app.has_prev_page());
    }

    #[tokio::test]
    async fn middle_page() {
        let app = test_app(vec![], 300, 100, 2).await;
        assert!(app.has_next_page());
        assert!(app.has_prev_page());
    }

    // ── clamp_selection ──

    #[tokio::test]
    async fn clamp_selection_empty_list() {
        let mut app = test_app(vec![], 0, 100, 1).await;
        app.selected_index = 5;
        app.clamp_selection();
        assert_eq!(app.selected_index, 0);
    }

    #[tokio::test]
    async fn clamp_selection_beyond_end() {
        let mut app = test_app(make_links(3), 3, 100, 1).await;
        app.selected_index = 10;
        app.clamp_selection();
        assert_eq!(app.selected_index, 2);
    }

    #[tokio::test]
    async fn clamp_selection_valid_index() {
        let mut app = test_app(make_links(5), 5, 100, 1).await;
        app.selected_index = 2;
        app.clamp_selection();
        assert_eq!(app.selected_index, 2);
    }

    #[tokio::test]
    async fn clamp_selection_at_boundary() {
        let mut app = test_app(make_links(5), 5, 100, 1).await;
        app.selected_index = 4;
        app.clamp_selection();
        assert_eq!(app.selected_index, 4);
    }

    #[tokio::test]
    async fn clamp_selection_one_past_end() {
        let mut app = test_app(make_links(5), 5, 100, 1).await;
        app.selected_index = 5;
        app.clamp_selection();
        assert_eq!(app.selected_index, 4);
    }

    // ── apply_sort ──

    #[tokio::test]
    async fn sort_by_code_ascending() {
        let links = vec![
            make_link("ccc", "https://c.com", 0),
            make_link("aaa", "https://a.com", 0),
            make_link("bbb", "https://b.com", 0),
        ];
        let mut app = test_app(links, 3, 100, 1).await;
        app.sort_column = Some(SortColumn::Code);
        app.sort_ascending = true;
        app.apply_sort();
        let codes: Vec<&str> = app.page_links.iter().map(|l| l.code.as_str()).collect();
        assert_eq!(codes, vec!["aaa", "bbb", "ccc"]);
    }

    #[tokio::test]
    async fn sort_by_code_descending() {
        let links = vec![
            make_link("aaa", "https://a.com", 0),
            make_link("ccc", "https://c.com", 0),
            make_link("bbb", "https://b.com", 0),
        ];
        let mut app = test_app(links, 3, 100, 1).await;
        app.sort_column = Some(SortColumn::Code);
        app.sort_ascending = false;
        app.apply_sort();
        let codes: Vec<&str> = app.page_links.iter().map(|l| l.code.as_str()).collect();
        assert_eq!(codes, vec!["ccc", "bbb", "aaa"]);
    }

    #[tokio::test]
    async fn sort_by_clicks_ascending() {
        let links = vec![
            make_link("a", "https://a.com", 5),
            make_link("b", "https://b.com", 1),
            make_link("c", "https://c.com", 10),
        ];
        let mut app = test_app(links, 3, 100, 1).await;
        app.sort_column = Some(SortColumn::Clicks);
        app.sort_ascending = true;
        app.apply_sort();
        let clicks: Vec<usize> = app.page_links.iter().map(|l| l.click).collect();
        assert_eq!(clicks, vec![1, 5, 10]);
    }

    #[tokio::test]
    async fn no_sort_preserves_order() {
        let links = vec![
            make_link("c", "https://c.com", 0),
            make_link("a", "https://a.com", 0),
            make_link("b", "https://b.com", 0),
        ];
        let mut app = test_app(links, 3, 100, 1).await;
        app.sort_column = None;
        app.apply_sort();
        let codes: Vec<&str> = app.page_links.iter().map(|l| l.code.as_str()).collect();
        assert_eq!(codes, vec!["c", "a", "b"]);
    }

    // ── adjust_scroll_offset ──

    #[tokio::test]
    async fn scroll_cursor_above_window() {
        let mut app = test_app(make_links(50), 50, 100, 1).await;
        app.last_visible_height = 10;
        app.scroll_offset = 20;
        app.selected_index = 15;
        app.adjust_scroll_offset();
        assert_eq!(app.scroll_offset, 15);
    }

    #[tokio::test]
    async fn scroll_cursor_below_window() {
        let mut app = test_app(make_links(50), 50, 100, 1).await;
        app.last_visible_height = 10;
        app.scroll_offset = 0;
        app.selected_index = 15;
        app.adjust_scroll_offset();
        assert_eq!(app.scroll_offset, 6); // 15 - 10 + 1
    }

    #[tokio::test]
    async fn scroll_cursor_in_window() {
        let mut app = test_app(make_links(50), 50, 100, 1).await;
        app.last_visible_height = 10;
        app.scroll_offset = 5;
        app.selected_index = 8;
        app.adjust_scroll_offset();
        assert_eq!(app.scroll_offset, 5);
    }

    // ── move_selection_up / down ──

    #[tokio::test]
    async fn move_up_from_top() {
        let mut app = test_app(make_links(5), 5, 100, 1).await;
        app.selected_index = 0;
        app.move_selection_up();
        assert_eq!(app.selected_index, 0);
    }

    #[tokio::test]
    async fn move_up_normal() {
        let mut app = test_app(make_links(5), 5, 100, 1).await;
        app.selected_index = 3;
        app.move_selection_up();
        assert_eq!(app.selected_index, 2);
    }

    #[tokio::test]
    async fn move_down_from_bottom() {
        let mut app = test_app(make_links(5), 5, 100, 1).await;
        app.selected_index = 4;
        app.move_selection_down();
        assert_eq!(app.selected_index, 4);
    }

    #[tokio::test]
    async fn move_down_normal() {
        let mut app = test_app(make_links(5), 5, 100, 1).await;
        app.selected_index = 2;
        app.move_selection_down();
        assert_eq!(app.selected_index, 3);
    }

    #[tokio::test]
    async fn move_down_empty_list() {
        let mut app = test_app(vec![], 0, 100, 1).await;
        app.move_selection_down();
        assert_eq!(app.selected_index, 0);
    }
}
