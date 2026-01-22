//! Action 系统
//!
//! 定义所有 TUI 可以执行的动作，用于组件间解耦通信

use super::app::CurrentScreen;

/// TUI 动作枚举
///
/// 组件通过返回 Action 来通知 App 需要执行的操作
/// App 处理 Action 并可能产生新的 Action（Action 链）
#[derive(Debug, Clone, PartialEq, Default)]
pub enum Action {
    // ========== 导航 ==========
    /// 向上移动选择
    MoveUp,
    /// 向下移动选择
    MoveDown,
    /// 向上翻页
    PageUp,
    /// 向下翻页
    PageDown,
    /// 跳转到顶部
    JumpTop,
    /// 跳转到底部
    JumpBottom,

    // ========== 屏幕切换 ==========
    /// 切换到指定屏幕
    SwitchScreen(CurrentScreen),
    /// 返回上一个屏幕（通常是 Main）
    GoBack,

    // ========== 链接操作 ==========
    /// 保存新链接
    SaveLink,
    /// 更新现有链接
    UpdateLink,
    /// 删除选中的链接
    DeleteLink,
    /// 刷新链接列表
    RefreshLinks,
    /// 进入添加链接模式
    EnterAddMode,
    /// 进入编辑链接模式
    EnterEditMode,
    /// 进入删除确认模式
    EnterDeleteMode,
    /// 查看链接详情
    ViewDetails,

    // ========== 搜索 ==========
    /// 进入搜索模式
    EnterSearchMode,
    /// 更新搜索查询
    UpdateSearch(String),
    /// 清除搜索
    ClearSearch,
    /// 执行搜索过滤
    FilterLinks,

    // ========== 表单输入 ==========
    /// 切换编辑字段（Tab）
    ToggleField,
    /// 切换强制覆盖选项（Space）
    ToggleOverwrite,
    /// 输入字符
    InputChar(char),
    /// 删除字符（Backspace）
    DeleteChar,
    /// 清空输入
    ClearInputs,

    // ========== 文件操作 ==========
    /// 进入导入导出菜单
    EnterExportImportMenu,
    /// 导出链接
    ExportLinks,
    /// 导入链接
    ImportLinks,
    /// 进入文件浏览器
    EnterFileBrowser,
    /// 选择文件/目录
    BrowserSelect,
    /// 浏览器向上导航
    BrowserUp,
    /// 浏览器向下导航
    BrowserDown,
    /// 进入目录或选择文件
    BrowserEnter,
    /// 输入导出文件名
    EnterExportFilename,

    // ========== 通知消息 ==========
    /// 显示状态消息（成功）
    ShowStatus(String),
    /// 显示错误消息
    ShowError(String),
    /// 清除所有消息
    ClearMessages,

    // ========== 系统 ==========
    /// 显示帮助
    ShowHelp,
    /// 请求退出
    RequestExit,
    /// 确认退出
    ConfirmExit,
    /// 取消退出
    CancelExit,
    /// 定时器 tick
    Tick,
    /// 退出程序
    Quit,
    /// 无操作
    #[default]
    Noop,
}

impl Action {
    /// 判断是否是无操作
    pub fn is_noop(&self) -> bool {
        matches!(self, Action::Noop)
    }

    /// 判断是否应该导致程序退出
    pub fn should_quit(&self) -> bool {
        matches!(self, Action::Quit)
    }
}
