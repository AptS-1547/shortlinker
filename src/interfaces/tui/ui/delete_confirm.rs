use ratatui::{
    Frame,
    layout::{Margin, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Paragraph, Wrap},
};

use super::widgets::Popup;
use crate::interfaces::tui::app::App;
use crate::interfaces::tui::constants::popup;

pub fn draw_delete_confirm_screen(frame: &mut Frame, app: &mut App, area: Rect) {
    if let Some(link) = app.get_selected_link() {
        let inner_area = Popup::new("Confirm Delete", popup::DELETE_CONFIRM)
            .theme_color(Color::Red)
            .margin(Margin::new(2, 2))
            .render(frame, area);

        let text = vec![
            Line::from(""),
            Line::from(vec![Span::styled(
                "WARNING: Are you sure you want to delete this link?",
                Style::default().fg(Color::Yellow).bold(),
            )]),
            Line::from(""),
            Line::from(vec![
                Span::styled("Code: ", Style::default().fg(Color::DarkGray)),
                Span::styled(&link.code, Style::default().fg(Color::Cyan).bold()),
            ]),
            Line::from(vec![
                Span::styled("URL: ", Style::default().fg(Color::DarkGray)),
                Span::styled(&link.target, Style::default().fg(Color::Blue)),
            ]),
            Line::from(""),
            Line::from(vec![Span::styled(
                "This action cannot be undone!",
                Style::default().fg(Color::Red).bold(),
            )]),
        ];

        let paragraph = Paragraph::new(text)
            .alignment(ratatui::layout::Alignment::Center)
            .wrap(Wrap { trim: false });

        frame.render_widget(paragraph, inner_area);
    }
}
