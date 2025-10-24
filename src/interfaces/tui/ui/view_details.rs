use chrono::Utc;
use ratatui::{
    Frame,
    layout::{Margin, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Paragraph},
};

use super::common::centered_rect;
use crate::interfaces::tui::app::App;

pub fn draw_view_details_screen(frame: &mut Frame, app: &App, area: Rect) {
    if let Some(link) = app.get_selected_link() {
        let popup_area = centered_rect(75, 65, area);

        frame.render_widget(Clear, popup_area);

        let block = Block::default()
            .title(format!("Link Details: {}", link.code))
            .title_style(Style::default().fg(Color::Cyan).bold())
            .borders(Borders::ALL)
            .border_type(BorderType::Double)
            .border_style(Style::default().fg(Color::Cyan));
        frame.render_widget(block, popup_area);

        let inner_area = popup_area.inner(Margin::new(2, 1));

        // Calculate time remaining
        let expiry_info = if let Some(expires_at) = link.expires_at {
            let now = Utc::now();
            if expires_at <= now {
                (
                    "EXPIRED".to_string(),
                    Style::default().fg(Color::Red).bold(),
                )
            } else {
                let duration = expires_at - now;
                let days = duration.num_days();
                let hours = duration.num_hours() % 24;
                let remaining = if days > 0 {
                    format!("{} days {} hours remaining", days, hours)
                } else {
                    format!("{} hours remaining", duration.num_hours())
                };
                (remaining, Style::default().fg(Color::Green))
            }
        } else {
            (
                "Never expires".to_string(),
                Style::default().fg(Color::Cyan),
            )
        };

        let details = vec![
            Line::from(""),
            Line::from(vec![
                Span::styled("Short Code:  ", Style::default().fg(Color::Yellow).bold()),
                Span::styled(&link.code, Style::default().fg(Color::Cyan).bold()),
            ]),
            Line::from(""),
            Line::from(vec![Span::styled(
                "Target URL:  ",
                Style::default().fg(Color::Yellow).bold(),
            )]),
            Line::from(vec![
                Span::styled("  ", Style::default()),
                Span::styled(&link.target, Style::default().fg(Color::Blue)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("Click Count: ", Style::default().fg(Color::Yellow).bold()),
                Span::styled(
                    format!("{}", link.click),
                    Style::default().fg(Color::Green).bold(),
                ),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("Password:    ", Style::default().fg(Color::Yellow).bold()),
                Span::styled(
                    if link.password.is_some() {
                        "Protected"
                    } else {
                        "None"
                    },
                    if link.password.is_some() {
                        Style::default().fg(Color::Red).bold()
                    } else {
                        Style::default().fg(Color::DarkGray)
                    },
                ),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("Created At:  ", Style::default().fg(Color::Yellow).bold()),
                Span::styled(
                    link.created_at.format("%Y-%m-%d %H:%M:%S").to_string(),
                    Style::default().fg(Color::White),
                ),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("Expires:     ", Style::default().fg(Color::Yellow).bold()),
                Span::styled(
                    if let Some(exp) = link.expires_at {
                        exp.format("%Y-%m-%d %H:%M:%S").to_string()
                    } else {
                        "Never".to_string()
                    },
                    Style::default().fg(Color::White),
                ),
            ]),
            Line::from(vec![
                Span::styled("             ", Style::default()),
                Span::styled(expiry_info.0, expiry_info.1),
            ]),
            Line::from(""),
            Line::from(""),
            Line::from(vec![Span::styled(
                "Press [q] or [Esc] to close",
                Style::default().fg(Color::DarkGray),
            )]),
        ];

        let details_para = Paragraph::new(details).alignment(ratatui::layout::Alignment::Left);
        frame.render_widget(details_para, inner_area);
    }
}
