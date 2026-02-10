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

pub fn draw_import_mode_screen(frame: &mut Frame, _app: &mut App, area: Rect) {
    let inner_area = Popup::new("Import Mode", popup::IMPORT_MODE)
        .theme_color(Color::Yellow)
        .margin(Margin::new(2, 2))
        .render(frame, area);

    let text = vec![
        Line::from(""),
        Line::from(vec![Span::styled(
            "Select Import Mode",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from(""),
        Line::from(vec![
            Span::styled("  [", Style::default().fg(Color::DarkGray)),
            Span::styled("s", Style::default().fg(Color::Green).bold()),
            Span::styled("]  ", Style::default().fg(Color::DarkGray)),
            Span::styled("Skip existing", Style::default().fg(Color::White)),
            Span::styled("  (default)", Style::default().fg(Color::DarkGray)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  [", Style::default().fg(Color::DarkGray)),
            Span::styled("o", Style::default().fg(Color::Red).bold()),
            Span::styled("]  ", Style::default().fg(Color::DarkGray)),
            Span::styled("Overwrite existing", Style::default().fg(Color::White)),
        ]),
        Line::from(""),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Press [Esc] to cancel",
            Style::default().fg(Color::DarkGray),
        )]),
    ];

    let paragraph = Paragraph::new(text).alignment(ratatui::layout::Alignment::Center);
    frame.render_widget(paragraph, inner_area);
}
