//! 通用输入框组件
//!
//! 用于表单中的文本输入，支持：
//! - 激活状态高亮
//! - 验证错误显示
//! - 字符计数
//! - 密码遮蔽

use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, BorderType, Borders, Paragraph},
};

use crate::interfaces::tui::constants::colors;

/// 输入框组件
///
/// 使用 Builder 模式配置各种选项
///
/// # 示例
///
/// ```rust,ignore
/// InputField::new("Short Code", &app.short_code_input)
///     .active(true)
///     .error(Some("Invalid code"))
///     .placeholder("Leave empty for random")
///     .render(frame, area);
/// ```
pub struct InputField<'a> {
    /// 字段标题
    title: &'a str,
    /// 输入值
    value: &'a str,
    /// 是否处于激活状态
    is_active: bool,
    /// 验证错误信息
    error: Option<&'a str>,
    /// 占位符文本
    placeholder: Option<&'a str>,
    /// 是否显示字符计数
    show_char_count: bool,
    /// 是否遮蔽输入（密码模式）
    masked: bool,
    /// 是否必填
    required: bool,
    /// 是否只读
    readonly: bool,
}

impl<'a> InputField<'a> {
    /// 创建新的输入框
    pub fn new(title: &'a str, value: &'a str) -> Self {
        Self {
            title,
            value,
            is_active: false,
            error: None,
            placeholder: None,
            show_char_count: true,
            masked: false,
            required: false,
            readonly: false,
        }
    }

    /// 设置激活状态
    pub fn active(mut self, active: bool) -> Self {
        self.is_active = active;
        self
    }

    /// 设置验证错误
    pub fn error(mut self, error: Option<&'a str>) -> Self {
        self.error = error;
        self
    }

    /// 设置占位符
    pub fn placeholder(mut self, placeholder: &'a str) -> Self {
        self.placeholder = Some(placeholder);
        self
    }

    /// 设置是否显示字符计数
    #[allow(dead_code)]
    pub fn char_count(mut self, show: bool) -> Self {
        self.show_char_count = show;
        self
    }

    /// 设置密码遮蔽模式
    pub fn masked(mut self) -> Self {
        self.masked = true;
        self
    }

    /// 设置为必填字段
    pub fn required(mut self) -> Self {
        self.required = true;
        self
    }

    /// 设置为只读
    pub fn readonly(mut self) -> Self {
        self.readonly = true;
        self
    }

    /// 计算所需的高度（输入框 + 错误行）
    pub fn height(&self) -> u16 {
        if self.error.is_some() {
            4 // 3 for input + 1 for error
        } else {
            3 // just input
        }
    }

    /// 获取显示的标题
    fn display_title(&self) -> String {
        let mut title = self.title.to_string();

        // 添加必填标记
        if self.required {
            title.push_str(" *");
        }

        // 添加字符计数
        if self.show_char_count && !self.value.is_empty() {
            title = format!("{} ({} chars)", title, self.value.len());
        }

        // 添加占位符提示
        if self.value.is_empty()
            && let Some(placeholder) = self.placeholder {
                title = format!("{} ({})", self.title, placeholder);
            }

        // 添加只读标记
        if self.readonly {
            title.push_str(" [readonly]");
        }

        title
    }

    /// 获取边框样式
    fn border_style(&self) -> Style {
        if self.readonly {
            Style::default().fg(colors::MUTED)
        } else if self.is_active {
            Style::default()
                .fg(colors::HIGHLIGHT_FG)
                .bg(colors::HIGHLIGHT_BG)
                .bold()
        } else {
            Style::default().fg(Color::White)
        }
    }

    /// 获取显示的值
    fn display_value(&self) -> String {
        if self.masked && !self.value.is_empty() {
            "*".repeat(self.value.len())
        } else {
            self.value.to_string()
        }
    }

    /// 渲染输入框
    ///
    /// # 参数
    ///
    /// - `frame`: 渲染帧
    /// - `area`: 渲染区域，高度应该是 3（无错误）或 4（有错误）
    pub fn render(&self, frame: &mut Frame, area: Rect) {
        // 分割区域：输入框 + 错误信息
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Length(1)])
            .split(area);

        // 渲染输入框
        let input = Paragraph::new(self.display_value()).block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title(self.display_title())
                .border_style(self.border_style()),
        );
        frame.render_widget(input, chunks[0]);

        // 渲染错误信息
        if let Some(error) = self.error {
            let error_text = Paragraph::new(error).style(Style::default().fg(colors::ERROR));
            frame.render_widget(error_text, chunks[1]);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_field_title() {
        let field = InputField::new("Name", "test");
        assert!(field.display_title().contains("Name"));
        assert!(field.display_title().contains("4 chars"));

        let field = InputField::new("Name", "").required();
        assert!(field.display_title().contains("*"));

        let field = InputField::new("Name", "").placeholder("optional");
        assert!(field.display_title().contains("optional"));
    }

    #[test]
    fn test_input_field_masked() {
        let field = InputField::new("Password", "secret").masked();
        assert_eq!(field.display_value(), "******");
    }

    #[test]
    fn test_input_field_height() {
        let field = InputField::new("Name", "test");
        assert_eq!(field.height(), 3);

        let field = InputField::new("Name", "test").error(Some("Error"));
        assert_eq!(field.height(), 4);
    }
}
