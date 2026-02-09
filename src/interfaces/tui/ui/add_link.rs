use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::Paragraph,
};

use super::widgets::{InputField, Popup};
use crate::interfaces::tui::app::{App, EditingField};
use crate::interfaces::tui::constants::popup;

pub fn draw_add_link_screen(frame: &mut Frame, app: &mut App, area: Rect) {
    let inner_area = Popup::new("Add New Short Link", popup::ADD_LINK)
        .theme_color(Color::Green)
        .render(frame, area);

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

    // Short code input
    InputField::new("Short Code", &app.form.short_code)
        .active(matches!(
            app.form.currently_editing,
            Some(EditingField::ShortCode)
        ))
        .error(
            app.form
                .validation_errors
                .get("short_code")
                .map(|s| s.as_str()),
        )
        .placeholder("empty = random")
        .render(frame, chunks[0]);

    // Target URL input
    InputField::new("Target URL", &app.form.target_url)
        .active(matches!(
            app.form.currently_editing,
            Some(EditingField::TargetUrl)
        ))
        .error(
            app.form
                .validation_errors
                .get("target_url")
                .map(|s| s.as_str()),
        )
        .required()
        .render(frame, chunks[1]);

    // Expire time input
    InputField::new("Expire Time", &app.form.expire_time)
        .active(matches!(
            app.form.currently_editing,
            Some(EditingField::ExpireTime)
        ))
        .error(
            app.form
                .validation_errors
                .get("expire_time")
                .map(|s| s.as_str()),
        )
        .placeholder("e.g. 2024-12-31 or 7d")
        .render(frame, chunks[2]);

    // Password input
    InputField::new("Password", &app.form.password)
        .active(matches!(
            app.form.currently_editing,
            Some(EditingField::Password)
        ))
        .placeholder("optional")
        .masked()
        .render(frame, chunks[3]);

    // Force overwrite checkbox
    let force_text = if app.form.force_overwrite {
        "[x]"
    } else {
        "[ ]"
    };
    let force = Paragraph::new(Line::from(vec![
        Span::styled(force_text, Style::default().fg(Color::Green).bold()),
        Span::styled(
            " Force overwrite existing code (Space to toggle)",
            Style::default().fg(Color::DarkGray),
        ),
    ]));
    frame.render_widget(force, chunks[4]);
}
