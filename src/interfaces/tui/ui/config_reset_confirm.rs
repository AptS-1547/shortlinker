use ratatui::{
    Frame,
    layout::{Margin, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
};

use super::widgets::Popup;
use crate::interfaces::tui::app::App;
use crate::interfaces::tui::constants::popup;

pub fn draw_config_reset_confirm_screen(frame: &mut Frame, app: &mut App, area: Rect) {
    let inner_area = Popup::new("Reset Configuration", popup::CONFIG_RESET_CONFIRM)
        .theme_color(Color::Red)
        .margin(Margin::new(2, 2))
        .render(frame, area);

    // Look up the config to get the default value
    let config = app
        .system
        .configs
        .iter()
        .find(|c| c.key == app.system.config_edit_key);

    let default_value = config
        .map(|c| c.default_value.as_str())
        .unwrap_or("unknown");

    let text = vec![
        Line::from(""),
        Line::from(vec![Span::styled(
            "Reset to default value?",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  Key:     ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                app.system.config_edit_key.as_str(),
                Style::default().fg(Color::Cyan).bold(),
            ),
        ]),
        Line::from(vec![
            Span::styled("  Default: ", Style::default().fg(Color::DarkGray)),
            Span::styled(default_value, Style::default().fg(Color::Green)),
        ]),
        Line::from(""),
        Line::from(""),
        Line::from(vec![
            Span::styled("  [", Style::default().fg(Color::DarkGray)),
            Span::styled("y", Style::default().fg(Color::Green).bold()),
            Span::styled("] Yes    ", Style::default().fg(Color::DarkGray)),
            Span::styled("[", Style::default().fg(Color::DarkGray)),
            Span::styled("n", Style::default().fg(Color::Red).bold()),
            Span::styled("] No", Style::default().fg(Color::DarkGray)),
        ]),
    ];

    let paragraph = Paragraph::new(text).alignment(ratatui::layout::Alignment::Center);
    frame.render_widget(paragraph, inner_area);
}
