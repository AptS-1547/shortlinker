use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Paragraph},
};

use super::common::centered_rect;
use crate::interfaces::tui::app::{App, CurrentlyEditing};

pub fn draw_add_link_screen(frame: &mut Frame, app: &mut App, area: Rect) {
    let popup_area = centered_rect(80, 70, area);

    // Shadow effect
    let shadow = Block::default().style(Style::default().bg(Color::Black));
    frame.render_widget(shadow, popup_area);

    frame.render_widget(Clear, popup_area);

    let block = Block::default()
        .title("Add New Short Link")
        .title_style(Style::default().fg(Color::Green).bold())
        .borders(Borders::ALL)
        .border_type(BorderType::Double)
        .border_style(Style::default().fg(Color::Green));
    frame.render_widget(block, popup_area);

    let inner_area = popup_area.inner(Margin::new(2, 1));

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(4), // Short code + error
            Constraint::Length(4), // Target URL + error
            Constraint::Length(4), // Expire time + error
            Constraint::Length(4), // Password + error
            Constraint::Length(2), // Force overwrite
        ])
        .split(inner_area);

    // Short code input with character count
    let short_code_field = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Length(1)])
        .split(chunks[0]);

    let short_code_style = if matches!(app.currently_editing, Some(CurrentlyEditing::ShortCode)) {
        Style::default().fg(Color::Black).bg(Color::Yellow).bold()
    } else {
        Style::default().fg(Color::White)
    };

    let short_code_title = if app.short_code_input.is_empty() {
        "Short Code (empty = random)".to_string()
    } else {
        format!("Short Code ({} chars)", app.short_code_input.len())
    };

    let short_code = Paragraph::new(&*app.short_code_input).block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title(short_code_title)
            .border_style(short_code_style),
    );
    frame.render_widget(short_code, short_code_field[0]);

    // Short code validation error
    if let Some(error) = app.validation_errors.get("short_code") {
        let error_text = Paragraph::new(error.as_str()).style(Style::default().fg(Color::Red));
        frame.render_widget(error_text, short_code_field[1]);
    }

    // Target URL input
    let target_field = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Length(1)])
        .split(chunks[1]);

    let target_style = if matches!(app.currently_editing, Some(CurrentlyEditing::TargetUrl)) {
        Style::default().fg(Color::Black).bg(Color::Yellow).bold()
    } else {
        Style::default().fg(Color::White)
    };

    let target_title = if app.target_url_input.is_empty() {
        "Target URL *".to_string()
    } else {
        format!("Target URL ({} chars)", app.target_url_input.len())
    };

    let target = Paragraph::new(&*app.target_url_input).block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title(target_title)
            .border_style(target_style),
    );
    frame.render_widget(target, target_field[0]);

    // Target URL validation error
    if let Some(error) = app.validation_errors.get("target_url") {
        let error_text = Paragraph::new(error.as_str()).style(Style::default().fg(Color::Red));
        frame.render_widget(error_text, target_field[1]);
    }

    // Expire time input
    let expire_field = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Length(1)])
        .split(chunks[2]);

    let expire_style = if matches!(app.currently_editing, Some(CurrentlyEditing::ExpireTime)) {
        Style::default().fg(Color::Black).bg(Color::Yellow).bold()
    } else {
        Style::default().fg(Color::White)
    };

    let expire = Paragraph::new(&*app.expire_time_input).block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title("Expire Time (e.g. 2024-12-31 or 7d)")
            .border_style(expire_style),
    );
    frame.render_widget(expire, expire_field[0]);

    // Expire time validation error
    if let Some(error) = app.validation_errors.get("expire_time") {
        let error_text = Paragraph::new(error.as_str()).style(Style::default().fg(Color::Red));
        frame.render_widget(error_text, expire_field[1]);
    }

    // Password input
    let password_field = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Length(1)])
        .split(chunks[3]);

    let password_style = if matches!(app.currently_editing, Some(CurrentlyEditing::Password)) {
        Style::default().fg(Color::Black).bg(Color::Yellow).bold()
    } else {
        Style::default().fg(Color::White)
    };

    let password_display = if app.password_input.is_empty() {
        String::new()
    } else {
        "*".repeat(app.password_input.len())
    };

    let password = Paragraph::new(password_display).block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title("Password (optional)")
            .border_style(password_style),
    );
    frame.render_widget(password, password_field[0]);

    // Force overwrite checkbox
    let force_text = if app.force_overwrite { "[x]" } else { "[ ]" };
    let force = Paragraph::new(Line::from(vec![
        Span::styled(force_text, Style::default().fg(Color::Green).bold()),
        Span::styled(
            " Force overwrite existing code (Space to toggle)",
            Style::default().fg(Color::DarkGray),
        ),
    ]));
    frame.render_widget(force, chunks[4]);
}
