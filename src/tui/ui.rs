use chrono::Utc;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{Block, Borders, BorderType, Clear, List, ListItem, Paragraph, Row, Table, Wrap},
};

use super::app::{App, CurrentScreen, CurrentlyEditing};

pub fn ui(frame: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Min(10),   // Main content
            Constraint::Length(3), // Status
            Constraint::Length(2), // Footer
        ])
        .split(frame.area());

    // Enhanced title with version and stats
    draw_title_bar(frame, app, chunks[0]);

    // Main content based on current screen
    match app.current_screen {
        CurrentScreen::Main => draw_main_screen(frame, app, chunks[1]),
        CurrentScreen::AddLink => draw_add_link_screen(frame, app, chunks[1]),
        CurrentScreen::EditLink => draw_edit_link_screen(frame, app, chunks[1]),
        CurrentScreen::DeleteConfirm => draw_delete_confirm_screen(frame, app, chunks[1]),
        CurrentScreen::ExportImport => draw_export_import_screen(frame, app, chunks[1]),
        CurrentScreen::Exiting => draw_exiting_screen(frame, chunks[1]),
        CurrentScreen::Search => draw_search_screen(frame, app, chunks[1]),
        CurrentScreen::Help => draw_help_screen(frame, chunks[1]),
        CurrentScreen::ViewDetails => draw_view_details_screen(frame, app, chunks[1]),
        CurrentScreen::FileBrowser => draw_file_browser_screen(frame, app, chunks[1]),
        CurrentScreen::ExportFileName => draw_export_filename_screen(frame, app, chunks[1]),
    }

    // Enhanced status bar
    draw_status_bar(frame, app, chunks[2]);

    // Enhanced footer with styled shortcuts
    draw_footer(frame, app, chunks[3]);
}

/// Draw title bar with version and statistics
fn draw_title_bar(frame: &mut Frame, app: &App, area: Rect) {
    let title_text = vec![
        Line::from(vec![
            Span::styled("Shortlinker TUI", Style::default().fg(Color::Cyan).bold()),
            Span::styled(" v0.2.3 ", Style::default().fg(Color::DarkGray)),
            Span::styled("| ", Style::default().fg(Color::DarkGray)),
            Span::styled(format!("Total: {} ", app.links.len()), Style::default().fg(Color::Yellow)),
        ]),
    ];

    let title = Paragraph::new(title_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::Cyan)),
        )
        .alignment(ratatui::layout::Alignment::Center);

    frame.render_widget(title, area);
}

/// Draw status bar
fn draw_status_bar(frame: &mut Frame, app: &App, area: Rect) {
    let (status_text, status_style) = if !app.error_message.is_empty() {
        (
            format!("[ERROR] {}", app.error_message),
            Style::default().fg(Color::White).bg(Color::Red).bold(),
        )
    } else if !app.status_message.is_empty() {
        (
            format!("[SUCCESS] {}", app.status_message),
            Style::default().fg(Color::Black).bg(Color::Green).bold(),
        )
    } else {
        (
            "Ready".to_string(),
            Style::default().fg(Color::Cyan),
        )
    };

    let status = Paragraph::new(status_text)
        .style(status_style)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
        )
        .alignment(ratatui::layout::Alignment::Center);

    frame.render_widget(status, area);
}

/// Draw footer with keyboard shortcuts
fn draw_footer(frame: &mut Frame, app: &App, area: Rect) {
    let shortcuts = match app.current_screen {
        CurrentScreen::Main => vec![
            ("Up/Down", "Navigate", Color::Cyan),
            ("/", "Search", Color::Cyan),
            ("v", "View", Color::Cyan),
            ("a", "Add", Color::Green),
            ("e", "Edit", Color::Yellow),
            ("d", "Delete", Color::Red),
            ("x", "Export / Import", Color::Magenta),
            ("?", "Help", Color::Blue),
            ("q", "Quit", Color::Magenta),
        ],
        CurrentScreen::AddLink | CurrentScreen::EditLink => vec![
            ("Tab", "Switch Field", Color::Cyan),
            ("Enter", "Save", Color::Green),
            ("Esc", "Cancel", Color::Red),
        ],
        CurrentScreen::DeleteConfirm | CurrentScreen::Exiting => vec![
            ("y", "Yes", Color::Green),
            ("n", "No", Color::Red),
        ],
        CurrentScreen::ExportImport => vec![
            ("e", "Export", Color::Green),
            ("i", "Import", Color::Yellow),
            ("Esc", "Back", Color::Red),
        ],
        CurrentScreen::Search => vec![
            ("Enter", "Apply", Color::Green),
            ("Esc", "Cancel", Color::Red),
        ],
        CurrentScreen::Help | CurrentScreen::ViewDetails => vec![
            ("q/Esc", "Close", Color::Red),
        ],
        CurrentScreen::FileBrowser => vec![
            ("Up/Down", "Navigate", Color::Cyan),
            ("Enter", "Select/Open", Color::Green),
            ("Esc", "Back", Color::Red),
        ],
        CurrentScreen::ExportFileName => vec![
            ("Enter", "Confirm", Color::Green),
            ("Esc", "Cancel", Color::Red),
        ],
    };

    let mut spans = Vec::new();
    for (i, (key, desc, color)) in shortcuts.iter().enumerate() {
        if i > 0 {
            spans.push(Span::styled(" | ", Style::default().fg(Color::DarkGray)));
        }
        spans.push(Span::styled(format!("[{}]", key), Style::default().fg(*color).bold()));
        spans.push(Span::styled(format!(" {}", desc), Style::default().fg(Color::White)));
    }

    let footer = Paragraph::new(Line::from(spans))
        .alignment(ratatui::layout::Alignment::Center);

    frame.render_widget(footer, area);
}

fn draw_main_screen(frame: &mut Frame, app: &mut App, area: Rect) {
    let display_links = app.get_display_links();

    if display_links.is_empty() {
        // Empty state or no search results
        let empty_text = if app.is_searching {
            vec![
                Line::from(""),
                Line::from(""),
                Line::from(vec![
                    Span::styled("No links match your search", Style::default().fg(Color::Gray).bold()),
                ]),
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
                Line::from(vec![
                    Span::styled("No short links found", Style::default().fg(Color::Gray).bold()),
                ]),
                Line::from(""),
                Line::from(vec![
                    Span::styled("Press ", Style::default().fg(Color::DarkGray)),
                    Span::styled("[a]", Style::default().fg(Color::Green).bold()),
                    Span::styled(" to create your first link", Style::default().fg(Color::DarkGray)),
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
                    .title_style(Style::default().fg(Color::Cyan))
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
        format!("Search Results ({} found) - \"{}\"", display_links.len(), app.search_input)
    } else {
        "Short Links".to_string()
    };

    let table = Table::new(
        rows,
        [
            Constraint::Length(15),
            Constraint::Min(30),
            Constraint::Length(8),
            Constraint::Length(18),
        ],
    )
    .header(header)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title(title)
            .title_style(Style::default().fg(Color::Cyan).bold())
    )
    .column_spacing(2);

    frame.render_widget(table, area);
}

fn draw_add_link_screen(frame: &mut Frame, app: &mut App, area: Rect) {
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
        let error_text = Paragraph::new(error.as_str())
            .style(Style::default().fg(Color::Red));
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
        let error_text = Paragraph::new(error.as_str())
            .style(Style::default().fg(Color::Red));
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
        let error_text = Paragraph::new(error.as_str())
            .style(Style::default().fg(Color::Red));
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

fn draw_edit_link_screen(frame: &mut Frame, app: &mut App, area: Rect) {
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
            "********".to_string()
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

fn draw_delete_confirm_screen(frame: &mut Frame, app: &mut App, area: Rect) {
    if let Some(link) = app.get_selected_link() {
        let popup_area = centered_rect(65, 45, area);

        // Shadow effect
        let shadow = Block::default().style(Style::default().bg(Color::Black));
        frame.render_widget(shadow, popup_area);

        frame.render_widget(Clear, popup_area);

        let block = Block::default()
            .title("Confirm Delete")
            .title_style(Style::default().fg(Color::Red).bold())
            .borders(Borders::ALL)
            .border_type(BorderType::Double)
            .border_style(Style::default().fg(Color::Red));
        frame.render_widget(block, popup_area);

        let inner_area = popup_area.inner(Margin::new(2, 2));

        let text = vec![
            Line::from(""),
            Line::from(vec![
                Span::styled("WARNING: Are you sure you want to delete this link?", Style::default().fg(Color::Yellow).bold()),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("Code: ", Style::default().fg(Color::DarkGray)),
                Span::styled(&link.code, Style::default().fg(Color::Cyan).bold()),
            ]),
            Line::from(vec![
                Span::styled("URL: ", Style::default().fg(Color::DarkGray)),
                Span::styled(&link.target, Style::default().fg(Color::Blue)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("This action cannot be undone!", Style::default().fg(Color::Red).bold()),
            ]),
        ];

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
            Constraint::Percentage(50), // Export section
            Constraint::Percentage(50), // Import section
        ])
        .split(area);

    // Export section
    let export = Paragraph::new(vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("Export Links", Style::default().fg(Color::Green).bold()),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Path: ", Style::default().fg(Color::DarkGray)),
            Span::styled(&app.export_path, Style::default().fg(Color::Cyan)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Press ", Style::default().fg(Color::DarkGray)),
            Span::styled("[e]", Style::default().fg(Color::Green).bold()),
            Span::styled(" to export all links as JSON", Style::default().fg(Color::DarkGray)),
        ]),
    ])
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::Green))
            .title("Export")
    )
    .alignment(ratatui::layout::Alignment::Center);
    frame.render_widget(export, chunks[0]);

    // Import section
    let import = Paragraph::new(vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("Import Links", Style::default().fg(Color::Yellow).bold()),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Path: ", Style::default().fg(Color::DarkGray)),
            Span::styled(&app.import_path, Style::default().fg(Color::Cyan)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Press ", Style::default().fg(Color::DarkGray)),
            Span::styled("[i]", Style::default().fg(Color::Yellow).bold()),
            Span::styled(" to import links from JSON", Style::default().fg(Color::DarkGray)),
        ]),
    ])
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::Yellow))
            .title("Import")
    )
    .alignment(ratatui::layout::Alignment::Center);
    frame.render_widget(import, chunks[1]);
}

fn draw_exiting_screen(frame: &mut Frame, area: Rect) {
    let popup_area = centered_rect(55, 30, area);

    // Shadow effect
    let shadow = Block::default().style(Style::default().bg(Color::Black));
    frame.render_widget(shadow, popup_area);

    frame.render_widget(Clear, popup_area);

    let block = Block::default()
        .title("Exit Confirmation")
        .title_style(Style::default().fg(Color::Magenta).bold())
        .borders(Borders::ALL)
        .border_type(BorderType::Double)
        .border_style(Style::default().fg(Color::Magenta));
    frame.render_widget(block, popup_area);

    let inner_area = popup_area.inner(Margin::new(2, 2));

    let text = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("Are you sure you want to exit?", Style::default().fg(Color::White).bold()),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Press [y] to quit, [n] to cancel", Style::default().fg(Color::DarkGray)),
        ]),
    ];

    let paragraph = Paragraph::new(text)
        .alignment(ratatui::layout::Alignment::Center);

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

/// Draw search screen
fn draw_search_screen(frame: &mut Frame, app: &App, area: Rect) {
    let popup_area = centered_rect(70, 30, area);

    frame.render_widget(Clear, popup_area);

    let block = Block::default()
        .title("Search Links")
        .title_style(Style::default().fg(Color::Cyan).bold())
        .borders(Borders::ALL)
        .border_type(BorderType::Double)
        .border_style(Style::default().fg(Color::Cyan));
    frame.render_widget(block, popup_area);

    let inner_area = popup_area.inner(Margin::new(2, 2));

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Search input
            Constraint::Min(3),    // Instructions
        ])
        .split(inner_area);

    // Search input box
    let search_input = Paragraph::new(&*app.search_input)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title(format!("Search ({} chars)", app.search_input.len()))
                .border_style(Style::default().fg(Color::Yellow).bold())
        );
    frame.render_widget(search_input, chunks[0]);

    // Instructions
    let instructions = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("Type to search in codes and URLs", Style::default().fg(Color::DarkGray)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("[Enter]", Style::default().fg(Color::Green).bold()),
            Span::styled(" Apply  ", Style::default().fg(Color::White)),
            Span::styled("[Esc]", Style::default().fg(Color::Red).bold()),
            Span::styled(" Cancel", Style::default().fg(Color::White)),
        ]),
    ];

    let inst_para = Paragraph::new(instructions)
        .alignment(ratatui::layout::Alignment::Center);
    frame.render_widget(inst_para, chunks[1]);
}

/// Draw help screen
fn draw_help_screen(frame: &mut Frame, area: Rect) {
    let popup_area = centered_rect(80, 85, area);

    frame.render_widget(Clear, popup_area);

    let block = Block::default()
        .title("Help - Keyboard Shortcuts")
        .title_style(Style::default().fg(Color::Cyan).bold())
        .borders(Borders::ALL)
        .border_type(BorderType::Double)
        .border_style(Style::default().fg(Color::Cyan));
    frame.render_widget(block, popup_area);

    let inner_area = popup_area.inner(Margin::new(2, 1));

    let help_text = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("NAVIGATION", Style::default().fg(Color::Yellow).bold()),
        ]),
        Line::from(vec![
            Span::styled("  Up/Down, j/k    ", Style::default().fg(Color::Cyan)),
            Span::styled("Navigate list", Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("  Home, g          ", Style::default().fg(Color::Cyan)),
            Span::styled("Jump to top", Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("  End, G           ", Style::default().fg(Color::Cyan)),
            Span::styled("Jump to bottom", Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("  PageUp/PageDown  ", Style::default().fg(Color::Cyan)),
            Span::styled("Scroll 10 items", Style::default().fg(Color::White)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("ACTIONS", Style::default().fg(Color::Yellow).bold()),
        ]),
        Line::from(vec![
            Span::styled("  a                ", Style::default().fg(Color::Green)),
            Span::styled("Add new link", Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("  e                ", Style::default().fg(Color::Yellow)),
            Span::styled("Edit selected link", Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("  d                ", Style::default().fg(Color::Red)),
            Span::styled("Delete selected link", Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("  Enter, v         ", Style::default().fg(Color::Cyan)),
            Span::styled("View link details", Style::default().fg(Color::White)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("SEARCH & UTILITY", Style::default().fg(Color::Yellow).bold()),
        ]),
        Line::from(vec![
            Span::styled("  /                ", Style::default().fg(Color::Cyan)),
            Span::styled("Search links", Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("  x                ", Style::default().fg(Color::Blue)),
            Span::styled("Export/Import", Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("  ?, h             ", Style::default().fg(Color::Cyan)),
            Span::styled("Show this help", Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("  q                ", Style::default().fg(Color::Magenta)),
            Span::styled("Quit application", Style::default().fg(Color::White)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("FORM EDITING", Style::default().fg(Color::Yellow).bold()),
        ]),
        Line::from(vec![
            Span::styled("  Tab              ", Style::default().fg(Color::Cyan)),
            Span::styled("Switch field", Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("  Enter            ", Style::default().fg(Color::Green)),
            Span::styled("Save changes", Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("  Esc              ", Style::default().fg(Color::Red)),
            Span::styled("Cancel", Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("  Space            ", Style::default().fg(Color::Cyan)),
            Span::styled("Toggle checkbox", Style::default().fg(Color::White)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("STATUS INDICATORS", Style::default().fg(Color::Yellow).bold()),
        ]),
        Line::from(vec![
            Span::styled("  LOCKED           ", Style::default().fg(Color::Cyan)),
            Span::styled("Password protected", Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("  ACTIVE           ", Style::default().fg(Color::Green)),
            Span::styled("Link is active", Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("  EXPIRING         ", Style::default().fg(Color::Yellow)),
            Span::styled("Expires in <24h", Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("  EXPIRED          ", Style::default().fg(Color::Red)),
            Span::styled("Link has expired", Style::default().fg(Color::White)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Press any key to close", Style::default().fg(Color::DarkGray)),
        ]),
    ];

    let help_para = Paragraph::new(help_text)
        .alignment(ratatui::layout::Alignment::Left);
    frame.render_widget(help_para, inner_area);
}

/// Draw view details screen
fn draw_view_details_screen(frame: &mut Frame, app: &App, area: Rect) {
    if let Some(link) = app.get_selected_link() {
        let popup_area = centered_rect(75, 65, area);

        frame.render_widget(Clear, popup_area);

        let block = Block::default()
            .title(format!("Link Details: {}", link.code))
            .title_style(Style::default().fg(Color::Cyan).bold())
            .borders(Borders::ALL)
            .border_type(BorderType::Double)
            .border_style(Style::default().fg(Color::Cyan));
        frame.render_widget(block, popup_area);

        let inner_area = popup_area.inner(Margin::new(2, 1));

        // Calculate time remaining
        let expiry_info = if let Some(expires_at) = link.expires_at {
            let now = Utc::now();
            if expires_at <= now {
                ("EXPIRED".to_string(), Style::default().fg(Color::Red).bold())
            } else {
                let duration = expires_at - now;
                let days = duration.num_days();
                let hours = duration.num_hours() % 24;
                let remaining = if days > 0 {
                    format!("{} days {} hours remaining", days, hours)
                } else {
                    format!("{} hours remaining", duration.num_hours())
                };
                (remaining, Style::default().fg(Color::Green))
            }
        } else {
            ("Never expires".to_string(), Style::default().fg(Color::Cyan))
        };

        let details = vec![
            Line::from(""),
            Line::from(vec![
                Span::styled("Short Code:  ", Style::default().fg(Color::Yellow).bold()),
                Span::styled(&link.code, Style::default().fg(Color::Cyan).bold()),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("Target URL:  ", Style::default().fg(Color::Yellow).bold()),
            ]),
            Line::from(vec![
                Span::styled("  ", Style::default()),
                Span::styled(&link.target, Style::default().fg(Color::Blue)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("Click Count: ", Style::default().fg(Color::Yellow).bold()),
                Span::styled(format!("{}", link.click), Style::default().fg(Color::Green).bold()),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("Password:    ", Style::default().fg(Color::Yellow).bold()),
                Span::styled(
                    if link.password.is_some() { "Protected" } else { "None" },
                    if link.password.is_some() {
                        Style::default().fg(Color::Red).bold()
                    } else {
                        Style::default().fg(Color::DarkGray)
                    }
                ),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("Created At:  ", Style::default().fg(Color::Yellow).bold()),
                Span::styled(
                    link.created_at.format("%Y-%m-%d %H:%M:%S").to_string(),
                    Style::default().fg(Color::White)
                ),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("Expires:     ", Style::default().fg(Color::Yellow).bold()),
                Span::styled(
                    if let Some(exp) = link.expires_at {
                        exp.format("%Y-%m-%d %H:%M:%S").to_string()
                    } else {
                        "Never".to_string()
                    },
                    Style::default().fg(Color::White)
                ),
            ]),
            Line::from(vec![
                Span::styled("             ", Style::default()),
                Span::styled(expiry_info.0, expiry_info.1),
            ]),
            Line::from(""),
            Line::from(""),
            Line::from(vec![
                Span::styled("Press [q] or [Esc] to close", Style::default().fg(Color::DarkGray)),
            ]),
        ];

        let details_para = Paragraph::new(details)
            .alignment(ratatui::layout::Alignment::Left);
        frame.render_widget(details_para, inner_area);
    }
}

/// Draw file browser screen for import
fn draw_file_browser_screen(frame: &mut Frame, app: &mut App, area: Rect) {
    let popup_area = centered_rect(80, 80, area);
    frame.render_widget(Clear, popup_area);

    let block = Block::default()
        .title(format!(" File Browser - {} ", app.current_dir.display()))
        .title_style(Style::default().fg(Color::Cyan).bold())
        .borders(Borders::ALL)
        .border_type(BorderType::Double)
        .border_style(Style::default().fg(Color::Cyan));
    frame.render_widget(block, popup_area);

    let inner_area = popup_area.inner(Margin::new(2, 1));

    // Create file list items
    let items: Vec<ListItem> = app.dir_entries.iter().enumerate().map(|(idx, path)| {
        let file_name = path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("..");

        let (icon, color) = if path.is_dir() {
            ("[DIR]", Color::Blue)
        } else {
            ("[FILE]", Color::Green)
        };

        let style = if idx == app.browser_selected_index {
            Style::default().fg(color).bg(Color::DarkGray).bold()
        } else {
            Style::default().fg(color)
        };

        let line = Line::from(vec![
            Span::styled(icon, style),
            Span::styled(" ", style),
            Span::styled(file_name, style),
        ]);

        ListItem::new(line)
    }).collect();

    let list = List::new(items)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title("Select JSON file to import (Up/Down to navigate, Enter to select)")
            .border_style(Style::default().fg(Color::Yellow)));

    frame.render_widget(list, inner_area);
}

/// Draw export filename input screen
fn draw_export_filename_screen(frame: &mut Frame, app: &App, area: Rect) {
    let popup_area = centered_rect(60, 30, area);
    frame.render_widget(Clear, popup_area);

    let block = Block::default()
        .title(" Export Links ")
        .title_style(Style::default().fg(Color::Cyan).bold())
        .borders(Borders::ALL)
        .border_type(BorderType::Double)
        .border_style(Style::default().fg(Color::Green));
    frame.render_widget(block, popup_area);

    let inner_area = popup_area.inner(Margin::new(2, 2));

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Instructions
            Constraint::Length(3),  // Filename input
            Constraint::Length(2),  // Preview
            Constraint::Min(1),     // Empty space
        ])
        .split(inner_area);

    // Instructions
    let instructions = Paragraph::new(vec![
        Line::from(vec![
            Span::styled("Enter filename for export (will add .json if missing)",
                Style::default().fg(Color::Gray)),
        ]),
    ]);
    frame.render_widget(instructions, chunks[0]);

    // Filename input
    let filename_input = Paragraph::new(&*app.export_filename_input)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title("Filename")
                .border_style(Style::default().fg(Color::Yellow).bold())
        );
    frame.render_widget(filename_input, chunks[1]);

    // Preview
    let preview_text = if app.export_filename_input.is_empty() {
        "No filename entered".to_string()
    } else if app.export_filename_input.ends_with(".json") {
        format!("Will save as: {}", app.export_filename_input)
    } else {
        format!("Will save as: {}.json", app.export_filename_input)
    };

    let preview = Paragraph::new(vec![
        Line::from(vec![
            Span::styled(preview_text, Style::default().fg(Color::Cyan)),
        ]),
    ]);
    frame.render_widget(preview, chunks[2]);
}
