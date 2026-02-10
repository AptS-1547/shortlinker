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

/// Format seconds into a human-readable duration string (e.g. "2d 5h 30m 15s")
fn format_uptime(secs: u64) -> String {
    let days = secs / 86400;
    let hours = (secs % 86400) / 3600;
    let minutes = (secs % 3600) / 60;
    let seconds = secs % 60;

    let mut parts = Vec::new();
    if days > 0 {
        parts.push(format!("{}d", days));
    }
    if hours > 0 {
        parts.push(format!("{}h", hours));
    }
    if minutes > 0 {
        parts.push(format!("{}m", minutes));
    }
    parts.push(format!("{}s", seconds));
    parts.join(" ")
}

pub fn draw_server_status_screen(frame: &mut Frame, app: &mut App, area: Rect) {
    let inner_area = Popup::new("Server Status", popup::SERVER_STATUS)
        .theme_color(Color::Cyan)
        .margin(Margin::new(2, 2))
        .render(frame, area);

    // PLACEHOLDER_STATUS_BODY
    let text = if let Some(ref error) = app.system.status_error {
        vec![
            Line::from(""),
            Line::from(vec![Span::styled(
                "ERROR",
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            )]),
            Line::from(""),
            Line::from(vec![Span::styled(
                error.as_str(),
                Style::default().fg(Color::Red),
            )]),
            Line::from(""),
            Line::from(""),
            Line::from(vec![Span::styled(
                "Press [r] to retry, [Esc] to go back",
                Style::default().fg(Color::DarkGray),
            )]),
        ]
    } else {
        // PLACEHOLDER_STATUS_OK
        build_status_lines(app)
    };

    let paragraph = Paragraph::new(text).alignment(ratatui::layout::Alignment::Left);
    frame.render_widget(paragraph, inner_area);
}

fn build_status_lines(app: &App) -> Vec<Line<'_>> {
    let reload_status = if app.system.status_is_reloading {
        ("Reloading...", Color::Yellow)
    } else {
        ("Idle", Color::Green)
    };

    let last_data = app
        .system
        .status_last_data_reload
        .as_deref()
        .unwrap_or("Never");
    let last_config = app
        .system
        .status_last_config_reload
        .as_deref()
        .unwrap_or("Never");

    vec![
        Line::from(""),
        Line::from(vec![Span::styled(
            "SERVER INFORMATION",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  Version:       ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                app.system.status_version.as_str(),
                Style::default().fg(Color::Cyan).bold(),
            ),
        ]),
        Line::from(vec![
            Span::styled("  Uptime:        ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format_uptime(app.system.status_uptime_secs),
                Style::default().fg(Color::Green),
            ),
        ]),
        Line::from(vec![
            Span::styled("  Links Count:   ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("{}", app.system.status_links_count),
                Style::default().fg(Color::White).bold(),
            ),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "RELOAD STATUS",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  Status:        ", Style::default().fg(Color::DarkGray)),
            Span::styled(reload_status.0, Style::default().fg(reload_status.1)),
        ]),
        Line::from(vec![
            Span::styled("  Last Data:     ", Style::default().fg(Color::DarkGray)),
            Span::styled(last_data, Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("  Last Config:   ", Style::default().fg(Color::DarkGray)),
            Span::styled(last_config, Style::default().fg(Color::White)),
        ]),
        Line::from(""),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Press [r] to refresh, [Esc] to go back",
            Style::default().fg(Color::DarkGray),
        )]),
    ]
}
