use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
};

use super::widgets::{InputField, Popup};
use crate::interfaces::tui::app::{App, PasswordField};
use crate::interfaces::tui::constants::popup;

pub fn draw_password_reset_screen(frame: &mut Frame, app: &mut App, area: Rect) {
    let inner_area = Popup::new("Reset Admin Password", popup::PASSWORD_RESET)
        .theme_color(Color::Red)
        .margin(Margin::new(2, 1))
        .render(frame, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Hint text
            Constraint::Length(4), // Password field
            Constraint::Length(4), // Confirm field
            Constraint::Length(2), // Error message
            Constraint::Min(1),    // Footer
        ])
        .split(inner_area);

    // Hint
    let hint = Paragraph::new(vec![
        Line::from(""),
        Line::from(vec![Span::styled(
            "Min 8 characters. Use [Tab] to switch fields.",
            Style::default().fg(Color::DarkGray),
        )]),
    ]);
    frame.render_widget(hint, chunks[0]);

    // Password field
    let is_password_active = matches!(app.system.password_field, Some(PasswordField::Password));
    InputField::new("New Password", &app.system.password_input)
        .active(is_password_active)
        .masked()
        .required()
        .render(frame, chunks[1]);

    // Confirm field
    let is_confirm_active = matches!(app.system.password_field, Some(PasswordField::Confirm));
    InputField::new("Confirm Password", &app.system.password_confirm)
        .active(is_confirm_active)
        .masked()
        .required()
        .render(frame, chunks[2]);

    // Error message
    if let Some(ref error) = app.system.password_error {
        let error_text = Paragraph::new(Line::from(vec![Span::styled(
            error.as_str(),
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        )]));
        frame.render_widget(error_text, chunks[3]);
    }

    // Footer
    let footer = Paragraph::new(Line::from(vec![Span::styled(
        "Press [Enter] to save, [Esc] to cancel",
        Style::default().fg(Color::DarkGray),
    )]));
    frame.render_widget(footer, chunks[4]);
}
