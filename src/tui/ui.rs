use chrono::Utc;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
};

use super::app::{App, CurrentScreen, CurrentlyEditing};

pub fn ui(frame: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Min(10),   // Main content
            Constraint::Length(3), // Status
            Constraint::Length(1), // Footer
        ])
        .split(frame.area());

    // Title
    let title = Paragraph::new(Text::styled(
        "🔗 Shortlinker TUI - URL Shortener Manager",
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    ))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Blue)),
    )
    .alignment(ratatui::layout::Alignment::Center);

    frame.render_widget(title, chunks[0]);

    // Main content based on current screen
    match app.current_screen {
        CurrentScreen::Main => draw_main_screen(frame, app, chunks[1]),
        CurrentScreen::AddLink => draw_add_link_screen(frame, app, chunks[1]),
        CurrentScreen::EditLink => draw_edit_link_screen(frame, app, chunks[1]),
        CurrentScreen::DeleteConfirm => draw_delete_confirm_screen(frame, app, chunks[1]),
        CurrentScreen::ExportImport => draw_export_import_screen(frame, app, chunks[1]),
        CurrentScreen::Exiting => draw_exiting_screen(frame, chunks[1]),
    }

    // Status bar
    let status_text = if !app.error_message.is_empty() {
        Span::styled(&app.error_message, Style::default().fg(Color::Red))
    } else if !app.status_message.is_empty() {
        Span::styled(&app.status_message, Style::default().fg(Color::Green))
    } else {
        Span::styled("Ready", Style::default().fg(Color::Gray))
    };

    let status = Paragraph::new(Line::from(status_text))
        .block(Block::default().borders(Borders::ALL))
        .alignment(ratatui::layout::Alignment::Center);

    frame.render_widget(status, chunks[2]);

    // Footer with key hints
    let footer_text = match app.current_screen {
        CurrentScreen::Main => {
            "↑↓ Navigate | (a)dd | (e)dit | (d)elete | (x)port/(i)mport | (q)uit"
        }
        CurrentScreen::AddLink | CurrentScreen::EditLink => {
            "(Tab) Switch field | (Enter) Save | (Esc) Cancel"
        }
        CurrentScreen::DeleteConfirm => "(y)es | (n)o",
        CurrentScreen::ExportImport => "(e)xport | (i)mport | (Esc) Back",
        CurrentScreen::Exiting => "(y)es | (n)o",
    };

    let footer = Paragraph::new(Line::from(Span::styled(
        footer_text,
        Style::default().fg(Color::Yellow),
    )))
    .alignment(ratatui::layout::Alignment::Center);

    frame.render_widget(footer, chunks[3]);
}

fn draw_main_screen(frame: &mut Frame, app: &mut App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(100)])
        .split(area);

    let mut list_items = Vec::new();

    if app.links.is_empty() {
        list_items.push(ListItem::new(Line::from(Span::styled(
            "No short links found. Press 'a' to add one.",
            Style::default().fg(Color::Gray),
        ))));
    } else {
        for (i, (code, link)) in app.links.iter().enumerate() {
            let mut parts = vec![
                Span::styled(format!("{: <15}", code), Style::default().fg(Color::Cyan)),
                Span::styled(" → ", Style::default().fg(Color::White)),
                Span::styled(&link.target, Style::default().fg(Color::Blue)),
            ];

            if let Some(expires_at) = link.expires_at {
                if expires_at > Utc::now() {
                    parts.push(Span::styled(
                        format!(" (expires: {})", expires_at.format("%Y-%m-%d %H:%M")),
                        Style::default().fg(Color::Yellow),
                    ));
                } else {
                    parts.push(Span::styled(" (EXPIRED)", Style::default().fg(Color::Red)));
                }
            }

            if link.password.is_some() {
                parts.push(Span::styled(" 🔒", Style::default().fg(Color::Magenta)));
            }

            if link.click > 0 {
                parts.push(Span::styled(
                    format!(" ({} clicks)", link.click),
                    Style::default().fg(Color::Green),
                ));
            }

            let style = if i == app.selected_index {
                Style::default().bg(Color::DarkGray)
            } else {
                Style::default()
            };

            list_items.push(ListItem::new(Line::from(parts)).style(style));
        }
    }

    let list = List::new(list_items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(format!("Short Links ({})", app.links.len()))
            .title_style(
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
    );

    frame.render_widget(list, chunks[0]);
}

fn draw_add_link_screen(frame: &mut Frame, app: &mut App, area: Rect) {
    let popup_area = centered_rect(80, 70, area);
    frame.render_widget(Clear, popup_area);

    let block = Block::default()
        .title("Add New Short Link")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Green));
    frame.render_widget(block, popup_area);

    let inner_area = popup_area.inner(Margin::new(1, 1));

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Short code
            Constraint::Length(3), // Target URL
            Constraint::Length(3), // Expire time
            Constraint::Length(3), // Password
            Constraint::Length(3), // Force overwrite
        ])
        .split(inner_area);

    // Short code input
    let short_code_style = if matches!(app.currently_editing, Some(CurrentlyEditing::ShortCode)) {
        Style::default().fg(Color::Black).bg(Color::Yellow)
    } else {
        Style::default().fg(Color::White)
    };

    let short_code = Paragraph::new(&*app.short_code_input).block(
        Block::default()
            .borders(Borders::ALL)
            .title("Short Code (leave empty for random)")
            .border_style(short_code_style),
    );
    frame.render_widget(short_code, chunks[0]);

    // Target URL input
    let target_style = if matches!(app.currently_editing, Some(CurrentlyEditing::TargetUrl)) {
        Style::default().fg(Color::Black).bg(Color::Yellow)
    } else {
        Style::default().fg(Color::White)
    };

    let target = Paragraph::new(&*app.target_url_input).block(
        Block::default()
            .borders(Borders::ALL)
            .title("Target URL")
            .border_style(target_style),
    );
    frame.render_widget(target, chunks[1]);

    // Expire time input
    let expire_style = if matches!(app.currently_editing, Some(CurrentlyEditing::ExpireTime)) {
        Style::default().fg(Color::Black).bg(Color::Yellow)
    } else {
        Style::default().fg(Color::White)
    };

    let expire = Paragraph::new(&*app.expire_time_input).block(
        Block::default()
            .borders(Borders::ALL)
            .title("Expire Time (optional, e.g., 2024-12-31T23:59:59Z or 1d)")
            .border_style(expire_style),
    );
    frame.render_widget(expire, chunks[2]);

    // Password input
    let password_style = if matches!(app.currently_editing, Some(CurrentlyEditing::Password)) {
        Style::default().fg(Color::Black).bg(Color::Yellow)
    } else {
        Style::default().fg(Color::White)
    };

    let password_display = if app.password_input.is_empty() {
        String::new()
    } else {
        "•".repeat(app.password_input.len())
    };

    let password = Paragraph::new(password_display).block(
        Block::default()
            .borders(Borders::ALL)
            .title("Password (optional)")
            .border_style(password_style),
    );
    frame.render_widget(password, chunks[3]);

    // Force overwrite checkbox
    let force_text = if app.force_overwrite { "[✓]" } else { "[ ]" };
    let force = Paragraph::new(Line::from(vec![
        Span::styled(force_text, Style::default().fg(Color::Green)),
        Span::styled(
            " Force overwrite existing code",
            Style::default().fg(Color::White),
        ),
    ]))
    .block(Block::default().borders(Borders::ALL).title("Options"));
    frame.render_widget(force, chunks[4]);
}

fn draw_edit_link_screen(frame: &mut Frame, app: &mut App, area: Rect) {
    if let Some(link) = app.get_selected_link() {
        let popup_area = centered_rect(80, 70, area);
        frame.render_widget(Clear, popup_area);

        let block = Block::default()
            .title(format!("Edit Link: {}", link.code))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow));
        frame.render_widget(block, popup_area);

        let inner_area = popup_area.inner(Margin::new(1, 1));

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
                .title("Short Code (read-only)")
                .border_style(Style::default().fg(Color::Gray)),
        );
        frame.render_widget(short_code, chunks[0]);

        // Target URL input
        let target_style = if matches!(app.currently_editing, Some(CurrentlyEditing::TargetUrl)) {
            Style::default().fg(Color::Black).bg(Color::Yellow)
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
                .title("Target URL")
                .border_style(target_style),
        );
        frame.render_widget(target, chunks[1]);

        // Expire time input
        let expire_style = if matches!(app.currently_editing, Some(CurrentlyEditing::ExpireTime)) {
            Style::default().fg(Color::Black).bg(Color::Yellow)
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
                .title("Expire Time")
                .border_style(expire_style),
        );
        frame.render_widget(expire, chunks[2]);

        // Password input
        let password_style = if matches!(app.currently_editing, Some(CurrentlyEditing::Password)) {
            Style::default().fg(Color::Black).bg(Color::Yellow)
        } else {
            Style::default().fg(Color::White)
        };

        let password_text = if matches!(app.currently_editing, Some(CurrentlyEditing::Password)) {
            if app.password_input.is_empty() {
                String::new()
            } else {
                "•".repeat(app.password_input.len())
            }
        } else {
            if link.password.is_some() {
                "••••••••".to_string()
            } else {
                String::new()
            }
        };

        let password = Paragraph::new(password_text).block(
            Block::default()
                .borders(Borders::ALL)
                .title("Password (leave empty to keep current)")
                .border_style(password_style),
        );
        frame.render_widget(password, chunks[3]);
    }
}

fn draw_delete_confirm_screen(frame: &mut Frame, app: &mut App, area: Rect) {
    if let Some(link) = app.get_selected_link() {
        let popup_area = centered_rect(60, 40, area);
        frame.render_widget(Clear, popup_area);

        let block = Block::default()
            .title("Confirm Delete")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Red));
        frame.render_widget(block, popup_area);

        let inner_area = popup_area.inner(Margin::new(1, 1));

        let text = format!(
            "Are you sure you want to delete this short link?\n\nCode: {}\nTarget: {}\n\nThis action cannot be undone.",
            link.code, link.target
        );

        let paragraph = Paragraph::new(text)
            .alignment(ratatui::layout::Alignment::Center)
            .wrap(Wrap { trim: false });

        frame.render_widget(paragraph, inner_area);
    }
}

fn draw_export_import_screen(frame: &mut Frame, app: &mut App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5), // Export section
            Constraint::Length(5), // Import section
        ])
        .split(area);

    // Export section
    let export = Paragraph::new(vec![
        Line::from(Span::styled(
            "Export Links",
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(format!("Path: {}", app.export_path)),
        Line::from("(e) to export all links as JSON"),
    ])
    .block(Block::default().borders(Borders::ALL).title("Export"));
    frame.render_widget(export, chunks[0]);

    // Import section
    let import = Paragraph::new(vec![
        Line::from(Span::styled(
            "Import Links",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(format!("Path: {}", app.import_path)),
        Line::from("(i) to import links from JSON"),
    ])
    .block(Block::default().borders(Borders::ALL).title("Import"));
    frame.render_widget(import, chunks[1]);
}

fn draw_exiting_screen(frame: &mut Frame, area: Rect) {
    let popup_area = centered_rect(60, 25, area);
    frame.render_widget(Clear, popup_area);

    let block = Block::default()
        .title("Exit Confirmation")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Red));
    frame.render_widget(block, popup_area);

    let inner_area = popup_area.inner(Margin::new(1, 1));

    let text = "Would you like to exit the Shortlinker TUI?\n\n(y)es / (n)o";

    let paragraph = Paragraph::new(text).alignment(ratatui::layout::Alignment::Center);

    frame.render_widget(paragraph, inner_area);
}

/// helper function to create a centered rect using up certain percentage of the available rect `r`
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    // Cut the given rectangle into three vertical pieces
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    // Then cut the middle vertical piece into three width-wise pieces
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1] // Return the middle chunk
}
