use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph},
};

use crate::interfaces::tui::app::App;

pub fn draw_export_import_screen(frame: &mut Frame, app: &mut App, area: Rect) {
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
        Line::from(vec![Span::styled(
            "Export Links",
            Style::default().fg(Color::Green).bold(),
        )]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Path: ", Style::default().fg(Color::DarkGray)),
            Span::styled(&app.export_path, Style::default().fg(Color::Cyan)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Press ", Style::default().fg(Color::DarkGray)),
            Span::styled("[e]", Style::default().fg(Color::Green).bold()),
            Span::styled(
                " to export all links as JSON",
                Style::default().fg(Color::DarkGray),
            ),
        ]),
    ])
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::Green))
            .title("Export"),
    )
    .alignment(ratatui::layout::Alignment::Center);
    frame.render_widget(export, chunks[0]);

    // Import section
    let import = Paragraph::new(vec![
        Line::from(""),
        Line::from(vec![Span::styled(
            "Import Links",
            Style::default().fg(Color::Yellow).bold(),
        )]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Path: ", Style::default().fg(Color::DarkGray)),
            Span::styled(&app.import_path, Style::default().fg(Color::Cyan)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Press ", Style::default().fg(Color::DarkGray)),
            Span::styled("[i]", Style::default().fg(Color::Yellow).bold()),
            Span::styled(
                " to import links from JSON",
                Style::default().fg(Color::DarkGray),
            ),
        ]),
    ])
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::Yellow))
            .title("Import"),
    )
    .alignment(ratatui::layout::Alignment::Center);
    frame.render_widget(import, chunks[1]);
}
