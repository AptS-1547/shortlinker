use ratatui::{
    Frame,
    layout::{Margin, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::Paragraph,
};

use super::widgets::Popup;
use crate::interfaces::tui::constants::popup;

pub fn draw_exiting_screen(frame: &mut Frame, area: Rect) {
    let inner_area = Popup::new("Exit Confirmation", popup::EXITING)
        .theme_color(Color::Magenta)
        .margin(Margin::new(2, 2))
        .render(frame, area);

    let text = vec![
        Line::from(""),
        Line::from(vec![Span::styled(
            "Are you sure you want to exit?",
            Style::default().fg(Color::White).bold(),
        )]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Press [y] to quit, [n] to cancel",
            Style::default().fg(Color::DarkGray),
        )]),
    ];

    let paragraph = Paragraph::new(text).alignment(ratatui::layout::Alignment::Center);

    frame.render_widget(paragraph, inner_area);
}
