use ratatui::{
    Frame,
    layout::{Margin, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
};

use super::widgets::Popup;
use crate::interfaces::tui::app::App;
use crate::interfaces::tui::constants::popup;

pub fn draw_system_menu_screen(frame: &mut Frame, _app: &mut App, area: Rect) {
    let inner_area = Popup::new("System Operations", popup::SYSTEM_MENU)
        .theme_color(Color::Magenta)
        .margin(Margin::new(2, 2))
        .render(frame, area);

    let text = vec![
        Line::from(""),
        Line::from(vec![Span::styled(
            "System Management",
            Style::default()
                .fg(Color::Magenta)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from(""),
        Line::from(vec![
            Span::styled("  [", Style::default().fg(Color::DarkGray)),
            Span::styled("s", Style::default().fg(Color::Cyan).bold()),
            Span::styled("]  ", Style::default().fg(Color::DarkGray)),
            Span::styled("Server Status", Style::default().fg(Color::White)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  [", Style::default().fg(Color::DarkGray)),
            Span::styled("c", Style::default().fg(Color::Yellow).bold()),
            Span::styled("]  ", Style::default().fg(Color::DarkGray)),
            Span::styled("Runtime Configuration", Style::default().fg(Color::White)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  [", Style::default().fg(Color::DarkGray)),
            Span::styled("p", Style::default().fg(Color::Red).bold()),
            Span::styled("]  ", Style::default().fg(Color::DarkGray)),
            Span::styled("Reset Admin Password", Style::default().fg(Color::White)),
        ]),
        Line::from(""),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Press [Esc] to go back",
            Style::default().fg(Color::DarkGray),
        )]),
    ];

    let paragraph = Paragraph::new(text).alignment(ratatui::layout::Alignment::Center);
    frame.render_widget(paragraph, inner_area);
}
