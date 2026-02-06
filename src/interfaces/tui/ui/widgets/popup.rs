//! 通用弹窗容器
//!
//! 提供居中弹窗的辅助功能

use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Style},
    widgets::{Block, BorderType, Borders, Clear},
};

use crate::interfaces::tui::constants::PopupSize;

/// 弹窗容器
///
/// 提供居中弹窗的渲染辅助
pub struct Popup<'a> {
    /// 弹窗标题
    title: &'a str,
    /// 标题颜色
    title_color: Color,
    /// 边框颜色
    border_color: Color,
    /// 尺寸配置
    size: PopupSize,
    /// 内边距
    margin: Margin,
}

impl<'a> Popup<'a> {
    /// 创建新的弹窗
    pub fn new(title: &'a str, size: PopupSize) -> Self {
        Self {
            title,
            title_color: Color::Cyan,
            border_color: Color::Cyan,
            size,
            margin: Margin::new(2, 1),
        }
    }

    /// 设置标题颜色
    pub fn title_color(mut self, color: Color) -> Self {
        self.title_color = color;
        self
    }

    /// 设置边框颜色
    pub fn border_color(mut self, color: Color) -> Self {
        self.border_color = color;
        self
    }

    /// 设置主题颜色（同时设置标题和边框颜色）
    pub fn theme_color(mut self, color: Color) -> Self {
        self.title_color = color;
        self.border_color = color;
        self
    }

    /// 设置内边距
    pub fn margin(mut self, margin: Margin) -> Self {
        self.margin = margin;
        self
    }

    /// 渲染弹窗并返回内部区域
    ///
    /// 这个方法会：
    /// 1. 计算居中位置
    /// 2. 渲染阴影效果
    /// 3. 清除背景
    /// 4. 渲染边框
    /// 5. 返回内部可用区域
    pub fn render(&self, frame: &mut Frame, area: Rect) -> Rect {
        let popup_area = centered_rect(self.size.width, self.size.height, area);

        // 阴影效果
        let shadow = Block::default().style(Style::default().bg(Color::Black));
        frame.render_widget(shadow, popup_area);

        // 清除背景
        frame.render_widget(Clear, popup_area);

        // 边框
        let block = Block::default()
            .title(self.title)
            .title_style(Style::default().fg(self.title_color).bold())
            .borders(Borders::ALL)
            .border_type(BorderType::Double)
            .border_style(Style::default().fg(self.border_color));
        frame.render_widget(block, popup_area);

        // 返回内部区域
        popup_area.inner(self.margin)
    }
}

/// 创建居中矩形
///
/// 根据百分比在给定区域中创建居中的矩形
pub fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

/// 使用 PopupSize 创建居中矩形
#[allow(dead_code)]
pub fn centered_rect_from_size(size: PopupSize, r: Rect) -> Rect {
    centered_rect(size.width, size.height, r)
}
