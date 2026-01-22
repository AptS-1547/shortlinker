use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Paragraph},
};

use super::common::centered_rect;
use crate::interfaces::tui::app::App;

pub fn draw_export_filename_screen(frame: &mut Frame, app: &App, area: Rect) {
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
            Constraint::Length(3), // Instructions
            Constraint::Length(3), // Filename input
            Constraint::Length(2), // Preview
            Constraint::Min(1),    // Empty space
        ])
        .split(inner_area);

    // Instructions
    let instructions = Paragraph::new(vec![Line::from(vec![Span::styled(
        "Enter filename for export (will add .csv if missing)",
        Style::default().fg(Color::Gray),
    )])]);
    frame.render_widget(instructions, chunks[0]);

    // Filename input
    let filename_input = Paragraph::new(&*app.export_filename_input).block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title("Filename")
            .border_style(Style::default().fg(Color::Yellow).bold()),
    );
    frame.render_widget(filename_input, chunks[1]);

    // Preview
    let preview_text = if app.export_filename_input.is_empty() {
        "No filename entered".to_string()
    } else if app.export_filename_input.ends_with(".csv") {
        format!("Will save as: {}", app.export_filename_input)
    } else {
        format!("Will save as: {}.csv", app.export_filename_input)
    };

    let preview = Paragraph::new(vec![Line::from(vec![Span::styled(
        preview_text,
        Style::default().fg(Color::Cyan),
    )])]);
    frame.render_widget(preview, chunks[2]);
}
