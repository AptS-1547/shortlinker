use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Style},
    widgets::{Block, BorderType, Borders, Clear, Paragraph},
};

use super::common::centered_rect;
use crate::interfaces::tui::app::{App, CurrentlyEditing};

pub fn draw_edit_link_screen(frame: &mut Frame, app: &mut App, area: Rect) {
    if let Some(link) = app.get_selected_link() {
        let popup_area = centered_rect(80, 70, area);

        // Shadow effect
        let shadow = Block::default().style(Style::default().bg(Color::Black));
        frame.render_widget(shadow, popup_area);

        frame.render_widget(Clear, popup_area);

        let block = Block::default()
            .title(format!("Edit Link: {}", link.code))
            .title_style(Style::default().fg(Color::Yellow).bold())
            .borders(Borders::ALL)
            .border_type(BorderType::Double)
            .border_style(Style::default().fg(Color::Yellow));
        frame.render_widget(block, popup_area);

        let inner_area = popup_area.inner(Margin::new(2, 1));

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Short code (read-only)
                Constraint::Length(3), // Target URL
                Constraint::Length(3), // Expire time
                Constraint::Length(3), // Password
            ])
            .split(inner_area);

        // Short code (read-only)
        let short_code = Paragraph::new(link.code.as_str()).block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title("Short Code (read-only)")
                .border_style(Style::default().fg(Color::DarkGray)),
        );
        frame.render_widget(short_code, chunks[0]);

        // Target URL input
        let target_style = if matches!(app.currently_editing, Some(CurrentlyEditing::TargetUrl)) {
            Style::default().fg(Color::Black).bg(Color::Yellow).bold()
        } else {
            Style::default().fg(Color::White)
        };

        let target_text = if matches!(app.currently_editing, Some(CurrentlyEditing::TargetUrl)) {
            &app.target_url_input
        } else {
            &link.target
        };

        let target = Paragraph::new(target_text.as_str()).block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title("Target URL")
                .border_style(target_style),
        );
        frame.render_widget(target, chunks[1]);

        // Expire time input
        let expire_style = if matches!(app.currently_editing, Some(CurrentlyEditing::ExpireTime)) {
            Style::default().fg(Color::Black).bg(Color::Yellow).bold()
        } else {
            Style::default().fg(Color::White)
        };

        let expire_text = if matches!(app.currently_editing, Some(CurrentlyEditing::ExpireTime)) {
            &app.expire_time_input
        } else {
            &link.expires_at.map_or(String::new(), |dt| dt.to_rfc3339())
        };

        let expire = Paragraph::new(expire_text.as_str()).block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title("Expire Time")
                .border_style(expire_style),
        );
        frame.render_widget(expire, chunks[2]);

        // Password input
        let password_style = if matches!(app.currently_editing, Some(CurrentlyEditing::Password)) {
            Style::default().fg(Color::Black).bg(Color::Yellow).bold()
        } else {
            Style::default().fg(Color::White)
        };

        let password_text = if matches!(app.currently_editing, Some(CurrentlyEditing::Password)) {
            if app.password_input.is_empty() {
                String::new()
            } else {
                "*".repeat(app.password_input.len())
            }
        } else if link.password.is_some() {
            "[REDACTED]".to_string()
        } else {
            String::new()
        };

        let password = Paragraph::new(password_text).block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title("Password (empty = keep current)")
                .border_style(password_style),
        );
        frame.render_widget(password, chunks[3]);
    }
}
