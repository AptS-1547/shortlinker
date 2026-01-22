//! TUI 常量定义
//!
//! 集中管理所有 UI 相关的常量，避免魔法数字分散在代码各处

/// URL 显示截断长度
pub const URL_TRUNCATE_LENGTH: usize = 50;

/// 翻页滚动步长
pub const PAGE_SCROLL_STEP: usize = 10;

/// 短码最大长度
pub const MAX_SHORT_CODE_LENGTH: usize = 128;

/// 弹窗尺寸配置
#[derive(Debug, Clone, Copy)]
pub struct PopupSize {
    /// 宽度百分比 (0-100)
    pub width: u16,
    /// 高度百分比 (0-100)
    pub height: u16,
}

impl PopupSize {
    pub const fn new(width: u16, height: u16) -> Self {
        Self { width, height }
    }
}

/// 各弹窗的尺寸配置
pub mod popup {
    use super::PopupSize;

    /// 添加链接弹窗
    pub const ADD_LINK: PopupSize = PopupSize::new(80, 70);
    /// 编辑链接弹窗
    pub const EDIT_LINK: PopupSize = PopupSize::new(80, 70);
    /// 帮助弹窗
    pub const HELP: PopupSize = PopupSize::new(80, 85);
    /// 删除确认弹窗
    pub const DELETE_CONFIRM: PopupSize = PopupSize::new(65, 45);
    /// 搜索弹窗
    pub const SEARCH: PopupSize = PopupSize::new(70, 30);
    /// 查看详情弹窗
    pub const VIEW_DETAILS: PopupSize = PopupSize::new(70, 60);
    /// 导入导出菜单
    pub const EXPORT_IMPORT: PopupSize = PopupSize::new(50, 40);
    /// 文件浏览器
    pub const FILE_BROWSER: PopupSize = PopupSize::new(70, 70);
    /// 导出文件名输入
    pub const EXPORT_FILENAME: PopupSize = PopupSize::new(60, 30);
    /// 退出确认
    pub const EXITING: PopupSize = PopupSize::new(50, 25);
}

/// 颜色主题（预留扩展）
pub mod colors {
    use ratatui::style::Color;

    /// 主色调
    pub const PRIMARY: Color = Color::Cyan;
    /// 成功色
    pub const SUCCESS: Color = Color::Green;
    /// 警告色
    pub const WARNING: Color = Color::Yellow;
    /// 错误色
    pub const ERROR: Color = Color::Red;
    /// 次要文本色
    pub const MUTED: Color = Color::DarkGray;
    /// 高亮背景色
    pub const HIGHLIGHT_BG: Color = Color::Yellow;
    /// 高亮前景色
    pub const HIGHLIGHT_FG: Color = Color::Black;
}

/// 链接状态文本
pub mod status_text {
    /// 密码保护
    pub const LOCKED: &str = "LOCKED";
    /// 活跃状态
    pub const ACTIVE: &str = "ACTIVE";
    /// 即将过期（24小时内）
    pub const EXPIRING: &str = "EXPIRING";
    /// 已过期
    pub const EXPIRED: &str = "EXPIRED";
}
