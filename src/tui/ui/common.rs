use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph},
};

use crate::tui::app::{App, CurrentScreen};

/// Draw title bar with version and statistics
pub fn draw_title_bar(frame: &mut Frame, app: &App, area: Rect) {
    let title_text = vec![Line::from(vec![
        Span::styled("Shortlinker TUI", Style::default().fg(Color::Cyan).bold()),
        Span::styled(" v0.2.3 ", Style::default().fg(Color::DarkGray)),
        Span::styled("| ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            format!("Total: {} ", app.links.len()),
            Style::default().fg(Color::Yellow),
        ),
    ])];

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
pub fn draw_status_bar(frame: &mut Frame, app: &App, area: Rect) {
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
        ("Ready".to_string(), Style::default().fg(Color::Cyan))
    };

    let status = Paragraph::new(status_text)
        .style(status_style)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded),
        )
        .alignment(ratatui::layout::Alignment::Center);

    frame.render_widget(status, area);
}

/// Draw footer with keyboard shortcuts
pub fn draw_footer(frame: &mut Frame, app: &App, area: Rect) {
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
        CurrentScreen::DeleteConfirm | CurrentScreen::Exiting => {
            vec![("y", "Yes", Color::Green), ("n", "No", Color::Red)]
        }
        CurrentScreen::ExportImport => vec![
            ("e", "Export", Color::Green),
            ("i", "Import", Color::Yellow),
            ("Esc", "Back", Color::Red),
        ],
        CurrentScreen::Search => vec![
            ("Enter", "Apply", Color::Green),
            ("Esc", "Cancel", Color::Red),
        ],
        CurrentScreen::Help | CurrentScreen::ViewDetails => vec![("q/Esc", "Close", Color::Red)],
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
        spans.push(Span::styled(
            format!("[{}]", key),
            Style::default().fg(*color).bold(),
        ));
        spans.push(Span::styled(
            format!(" {}", desc),
            Style::default().fg(Color::White),
        ));
    }

    let footer = Paragraph::new(Line::from(spans)).alignment(ratatui::layout::Alignment::Center);

    frame.render_widget(footer, area);
}

/// Helper function to create a centered rect using up certain percentage of the available rect `r`
pub fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
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
