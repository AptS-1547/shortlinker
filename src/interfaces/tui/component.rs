//! Component trait 定义
//!
//! 基于 Ratatui Component Architecture 的组件接口

use ratatui::Frame;
use ratatui::crossterm::event::KeyCode;
use ratatui::layout::Rect;

use super::action::Action;

/// TUI 组件 trait
///
/// 每个组件封装自己的状态、事件处理和渲染逻辑。
/// 组件通过返回 Action 与外部通信，实现解耦。
///
/// # 生命周期
///
/// 1. `init()` - 组件初始化
/// 2. `handle_key()` - 处理键盘事件，返回 Action
/// 3. `update()` - 处理 Action，可能产生新的 Action
/// 4. `render()` - 渲染组件
///
/// # 示例
///
/// ```rust,ignore
/// struct MainScreen {
///     selected_index: usize,
/// }
///
/// impl Component for MainScreen {
///     fn handle_key(&mut self, key: KeyCode) -> Action {
///         match key {
///             KeyCode::Up => Action::MoveUp,
///             KeyCode::Down => Action::MoveDown,
///             KeyCode::Char('q') => Action::RequestExit,
///             _ => Action::Noop,
///         }
///     }
///
///     fn render(&mut self, frame: &mut Frame, area: Rect) {
///         // 渲染逻辑
///     }
/// }
/// ```
pub trait Component {
    /// 初始化组件
    ///
    /// 在组件首次使用前调用，用于设置初始状态或加载资源
    fn init(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }

    /// 处理键盘事件
    ///
    /// 返回一个 Action 表示需要执行的操作。
    /// 如果不需要处理该事件，返回 `Action::Noop`。
    fn handle_key(&mut self, _key: KeyCode) -> Action {
        Action::Noop
    }

    /// 处理 Action 更新
    ///
    /// 当组件需要响应某个 Action 时调用。
    /// 返回值可以是另一个 Action（实现 Action 链）或 `Action::Noop`。
    fn update(&mut self, _action: Action) -> Action {
        Action::Noop
    }

    /// 渲染组件
    ///
    /// 将组件渲染到指定的区域。这是唯一的必须实现的方法。
    fn render(&mut self, frame: &mut Frame, area: Rect);
}

/// 可聚焦的组件
///
/// 扩展 Component trait，添加焦点相关的方法
pub trait Focusable: Component {
    /// 组件是否获得焦点
    fn is_focused(&self) -> bool;

    /// 设置焦点状态
    fn set_focus(&mut self, focused: bool);
}
