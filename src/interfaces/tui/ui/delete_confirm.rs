use ratatui::{
    Frame,
    layout::{Margin, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Paragraph, Wrap},
};

use super::common::centered_rect;
use crate::interfaces::tui::app::App;

pub fn draw_delete_confirm_screen(frame: &mut Frame, app: &mut App, area: Rect) {
    if let Some(link) = app.get_selected_link() {
        let popup_area = centered_rect(65, 45, area);

        // Shadow effect
        let shadow = Block::default().style(Style::default().bg(Color::Black));
        frame.render_widget(shadow, popup_area);

        frame.render_widget(Clear, popup_area);

        let block = Block::default()
            .title("Confirm Delete")
            .title_style(Style::default().fg(Color::Red).bold())
            .borders(Borders::ALL)
            .border_type(BorderType::Double)
            .border_style(Style::default().fg(Color::Red));
        frame.render_widget(block, popup_area);

        let inner_area = popup_area.inner(Margin::new(2, 2));

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
