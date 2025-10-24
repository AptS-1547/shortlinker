use chrono::Utc;
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph, Row, Table},
};

use crate::tui::app::App;

pub fn draw_main_screen(frame: &mut Frame, app: &mut App, area: Rect) {
    let display_links = app.get_display_links();

    if display_links.is_empty() {
        // Empty state or no search results
        let empty_text = if app.is_searching {
            vec![
                Line::from(""),
                Line::from(""),
                Line::from(vec![Span::styled(
                    "No links match your search",
                    Style::default().fg(Color::Gray).bold(),
                )]),
                Line::from(""),
                Line::from(vec![
                    Span::styled("Search query: ", Style::default().fg(Color::DarkGray)),
                    Span::styled(&app.search_input, Style::default().fg(Color::Cyan).bold()),
                ]),
                Line::from(""),
                Line::from(vec![
                    Span::styled("Press ", Style::default().fg(Color::DarkGray)),
                    Span::styled("[Esc]", Style::default().fg(Color::Yellow).bold()),
                    Span::styled(" to clear search", Style::default().fg(Color::DarkGray)),
                ]),
            ]
        } else {
            vec![
                Line::from(""),
                Line::from(""),
                Line::from(vec![Span::styled(
                    "No short links found",
                    Style::default().fg(Color::Gray).bold(),
                )]),
                Line::from(""),
                Line::from(vec![
                    Span::styled("Press ", Style::default().fg(Color::DarkGray)),
                    Span::styled("[a]", Style::default().fg(Color::Green).bold()),
                    Span::styled(
                        " to create your first link",
                        Style::default().fg(Color::DarkGray),
                    ),
                ]),
            ]
        };

        let title = if app.is_searching {
            format!("Search Results ({})", app.search_input)
        } else {
            "Short Links".to_string()
        };

        let empty = Paragraph::new(empty_text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(Color::DarkGray))
                    .title(title)
                    .title_style(Style::default().fg(Color::Cyan)),
            )
            .alignment(ratatui::layout::Alignment::Center);

        frame.render_widget(empty, area);
        return;
    }

    // Table view for links
    let header = Row::new(vec!["Code", "URL", "Clicks", "Status"])
        .style(Style::default().fg(Color::Yellow).bold())
        .bottom_margin(1);

    let mut rows = Vec::new();
    for (i, (code, link)) in display_links.iter().enumerate() {
        // Truncate URL if too long
        let display_url = if link.target.len() > 50 {
            format!("{}...", &link.target[..50])
        } else {
            link.target.clone()
        };

        // Status indicators (text-based)
        let mut status_parts = Vec::new();

        // Password protection indicator
        if link.password.is_some() {
            status_parts.push("LOCKED");
        }

        // Expiration status
        if let Some(expires_at) = link.expires_at {
            let now = Utc::now();
            if expires_at <= now {
                status_parts.push("EXPIRED");
            } else if (expires_at - now).num_hours() < 24 {
                status_parts.push("EXPIRING");
            } else {
                status_parts.push("ACTIVE");
            }
        } else {
            status_parts.push("ACTIVE");
        }

        let status_text = status_parts.join(" ");

        let row_style = if i == app.selected_index {
            Style::default().bg(Color::DarkGray).fg(Color::White)
        } else {
            Style::default()
        };

        let row = Row::new(vec![
            Span::styled(code.to_string(), Style::default().fg(Color::Cyan).bold()),
            Span::styled(display_url, Style::default().fg(Color::Blue)),
            Span::styled(format!("{}", link.click), Style::default().fg(Color::Green)),
            Span::styled(status_text, Style::default().fg(Color::Yellow)),
        ])
        .style(row_style);

        rows.push(row);
    }

    let title = if app.is_searching {
        format!(
            "Search Results ({} found) - \"{}\"",
            display_links.len(),
            app.search_input
        )
    } else {
        "Short Links".to_string()
    };

    let table = Table::new(
        rows,
        [
            ratatui::layout::Constraint::Length(15),
            ratatui::layout::Constraint::Min(30),
            ratatui::layout::Constraint::Length(8),
            ratatui::layout::Constraint::Length(18),
        ],
    )
    .header(header)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title(title)
            .title_style(Style::default().fg(Color::Cyan).bold()),
    )
    .column_spacing(2);

    frame.render_widget(table, area);
}
