//! Detail panel component for displaying selected link information

use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph, Wrap},
};

use crate::interfaces::tui::app::App;

/// Draw the detail panel showing information about the selected link
pub fn draw_detail_panel(frame: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title("Details")
        .title_style(Style::default().fg(Color::Cyan).bold())
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::DarkGray));

    if let Some(link) = app.get_selected_link() {
        // Format expiration time
        let expires_text = link
            .expires_at
            .map(|dt| dt.format("%Y-%m-%d %H:%M").to_string())
            .unwrap_or_else(|| "Never".to_string());

        // Format created time
        let created_text = link.created_at.format("%Y-%m-%d %H:%M").to_string();

        // Check if link is active (not expired)
        let is_active = !link.is_expired();
        let status_text = if is_active { "Active" } else { "Expired" };
        let status_color = if is_active { Color::Green } else { Color::Red };

        // Check if password protected
        let password_text = if link.password.is_some() { "Yes" } else { "No" };

        let details = vec![
            Line::from(""),
            Line::from(vec![
                Span::styled("Code:      ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    &link.code,
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(""),
            Line::from(vec![Span::styled(
                "URL:       ",
                Style::default().fg(Color::DarkGray),
            )]),
            Line::from(vec![Span::styled(
                &link.target,
                Style::default().fg(Color::Blue),
            )]),
            Line::from(""),
            Line::from(vec![
                Span::styled("Clicks:    ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    format!("{}", link.click),
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("Status:    ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    status_text,
                    Style::default()
                        .fg(status_color)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("Created:   ", Style::default().fg(Color::DarkGray)),
                Span::styled(created_text, Style::default().fg(Color::White)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("Expires:   ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    expires_text,
                    Style::default().fg(if link.expires_at.is_some() {
                        Color::Yellow
                    } else {
                        Color::White
                    }),
                ),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("Password:  ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    password_text,
                    Style::default().fg(if link.password.is_some() {
                        Color::Yellow
                    } else {
                        Color::White
                    }),
                ),
            ]),
            Line::from(""),
            Line::from(""),
            Line::from(vec![Span::styled(
                "Actions:",
                Style::default().fg(Color::Gray),
            )]),
            Line::from(vec![
                Span::styled(
                    " y ",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled("Copy code", Style::default().fg(Color::DarkGray)),
            ]),
            Line::from(vec![
                Span::styled(
                    " Y ",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled("Copy URL", Style::default().fg(Color::DarkGray)),
            ]),
            Line::from(vec![
                Span::styled(
                    " e ",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled("Edit", Style::default().fg(Color::DarkGray)),
            ]),
            Line::from(vec![
                Span::styled(
                    " d ",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled("Delete", Style::default().fg(Color::DarkGray)),
            ]),
        ];

        let paragraph = Paragraph::new(details)
            .block(block)
            .wrap(Wrap { trim: true });

        frame.render_widget(paragraph, area);
    } else {
        // Empty state
        let empty_text = vec![
            Line::from(""),
            Line::from(""),
            Line::from(vec![Span::styled(
                "No link selected",
                Style::default().fg(Color::DarkGray),
            )]),
        ];

        let paragraph = Paragraph::new(empty_text)
            .block(block)
            .alignment(ratatui::layout::Alignment::Center);

        frame.render_widget(paragraph, area);
    }
}
