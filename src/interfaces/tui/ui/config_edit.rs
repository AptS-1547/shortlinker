use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
};

use super::widgets::{InputField, Popup};
use crate::interfaces::tui::app::App;
use crate::interfaces::tui::constants::popup;

pub fn draw_config_edit_screen(frame: &mut Frame, app: &mut App, area: Rect) {
    // Look up the config being edited
    let config = app
        .system
        .configs
        .iter()
        .find(|c| c.key == app.system.config_edit_key);

    let title = format!("Edit: {}", app.system.config_edit_key);

    let inner_area = Popup::new(&title, popup::CONFIG_EDIT)
        .theme_color(Color::Yellow)
        .margin(Margin::new(2, 1))
        .render(frame, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(8), // Info section
            Constraint::Length(4), // Input field + error
            Constraint::Length(2), // Error message
            Constraint::Min(1),    // Footer
        ])
        .split(inner_area);

    // Info section
    if let Some(cfg) = config {
        let mut info_lines = vec![
            Line::from(""),
            Line::from(vec![
                Span::styled("  Description: ", Style::default().fg(Color::DarkGray)),
                Span::styled(cfg.description.as_str(), Style::default().fg(Color::White)),
            ]),
            Line::from(vec![
                Span::styled("  Type:        ", Style::default().fg(Color::DarkGray)),
                Span::styled(cfg.value_type.as_str(), Style::default().fg(Color::Cyan)),
            ]),
            Line::from(vec![
                Span::styled("  Default:     ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    cfg.default_value.as_str(),
                    Style::default().fg(Color::Green),
                ),
            ]),
        ];

        // Show enum options if available
        if let Some(ref options) = cfg.enum_options {
            info_lines.push(Line::from(vec![
                Span::styled("  Allowed:     ", Style::default().fg(Color::DarkGray)),
                Span::styled(options.join(", "), Style::default().fg(Color::Magenta)),
            ]));
        }

        if cfg.requires_restart {
            info_lines.push(Line::from(vec![Span::styled(
                "  * Requires server restart to take effect",
                Style::default()
                    .fg(Color::Red)
                    .add_modifier(Modifier::ITALIC),
            )]));
        }

        let info = Paragraph::new(info_lines);
        frame.render_widget(info, chunks[0]);
    }

    // Input field
    InputField::new("New Value", &app.system.config_edit_value)
        .active(true)
        .error(app.system.config_edit_error.as_deref())
        .render(frame, chunks[1]);

    // Footer
    let footer = Paragraph::new(Line::from(vec![Span::styled(
        "Press [Enter] to save, [Esc] to cancel",
        Style::default().fg(Color::DarkGray),
    )]));
    frame.render_widget(footer, chunks[3]);
}
