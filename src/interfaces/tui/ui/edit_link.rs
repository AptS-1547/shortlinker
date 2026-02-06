use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::Color,
};

use super::widgets::{InputField, Popup};
use crate::interfaces::tui::app::{App, EditingField};
use crate::interfaces::tui::constants::popup;

pub fn draw_edit_link_screen(frame: &mut Frame, app: &mut App, area: Rect) {
    if let Some(link) = app.get_selected_link() {
        let inner_area = Popup::new(&format!("Edit Link: {}", link.code), popup::EDIT_LINK)
            .theme_color(Color::Yellow)
            .render(frame, area);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(4), // Short code (read-only) + space
                Constraint::Length(4), // Target URL + error
                Constraint::Length(4), // Expire time + error
                Constraint::Length(4), // Password + error
            ])
            .split(inner_area);

        // Short code (read-only)
        InputField::new("Short Code", &link.code)
            .readonly()
            .render(frame, chunks[0]);

        // Target URL input
        let is_editing_target = matches!(app.form.currently_editing, Some(EditingField::TargetUrl));
        let target_value = if is_editing_target {
            &app.form.target_url
        } else {
            &link.target
        };
        InputField::new("Target URL", target_value)
            .active(is_editing_target)
            .render(frame, chunks[1]);

        // Expire time input
        let is_editing_expire =
            matches!(app.form.currently_editing, Some(EditingField::ExpireTime));
        let expire_display = if is_editing_expire {
            app.form.expire_time.clone()
        } else {
            link.expires_at.map_or(String::new(), |dt| dt.to_rfc3339())
        };
        InputField::new("Expire Time", &expire_display)
            .active(is_editing_expire)
            .render(frame, chunks[2]);

        // Password input
        let is_editing_password =
            matches!(app.form.currently_editing, Some(EditingField::Password));
        let password_display = if is_editing_password {
            app.form.password.clone()
        } else if link.password.is_some() {
            "[REDACTED]".to_string()
        } else {
            String::new()
        };
        InputField::new("Password", &password_display)
            .active(is_editing_password)
            .placeholder("empty = keep current")
            .masked()
            .char_count(false) // 不显示字符计数，因为 [REDACTED] 会被误算
            .render(frame, chunks[3]);
    }
}
