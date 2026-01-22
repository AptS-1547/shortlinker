//! Batch delete confirmation popup

use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Paragraph},
};

use super::common::centered_rect;
use crate::interfaces::tui::app::App;

/// Draw the batch delete confirmation popup
pub fn draw_batch_delete_confirm_screen(frame: &mut Frame, app: &App, area: Rect) {
    let count = app.selected_items.len();

    // Use centered_rect like other popups
    let popup_area = centered_rect(65, 45, area);

    // Clear the background
    frame.render_widget(Clear, popup_area);

    let title = format!(" Delete {} Links ", count);

    let block = Block::default()
        .title(title)
        .title_style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Red));

    let inner_area = block.inner(popup_area);
    frame.render_widget(block, popup_area);

    // Content
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2), // Warning message
            Constraint::Length(2), // Link count
            Constraint::Min(1),    // Spacer
            Constraint::Length(2), // Instructions
        ])
        .split(inner_area);

    // Warning message
    let warning = Paragraph::new(vec![Line::from(vec![
        Span::styled(
            "Are you sure you want to delete ",
            Style::default().fg(Color::White),
        ),
        Span::styled(
            format!("{}", count),
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        ),
        Span::styled(" selected links?", Style::default().fg(Color::White)),
    ])])
    .alignment(Alignment::Center);

    frame.render_widget(warning, chunks[0]);

    // Show some of the selected codes
    let selected_codes: Vec<&String> = app.selected_items.iter().take(3).collect();
    let codes_preview = if app.selected_items.len() > 3 {
        let codes_str = selected_codes
            .iter()
            .map(|s| s.as_str())
            .collect::<Vec<_>>()
            .join(", ");
        format!("{}... and {} more", codes_str, app.selected_items.len() - 3)
    } else {
        selected_codes
            .iter()
            .map(|s| s.as_str())
            .collect::<Vec<_>>()
            .join(", ")
    };

    let codes_line = Paragraph::new(vec![Line::from(vec![Span::styled(
        codes_preview,
        Style::default().fg(Color::DarkGray),
    )])])
    .alignment(Alignment::Center);

    frame.render_widget(codes_line, chunks[1]);

    // Instructions
    let instructions = Paragraph::new(vec![Line::from(vec![
        Span::styled(
            "[Y] ",
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        ),
        Span::styled("Delete All  ", Style::default().fg(Color::White)),
        Span::styled(
            "[N] ",
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("Cancel", Style::default().fg(Color::White)),
    ])])
    .alignment(Alignment::Center);

    frame.render_widget(instructions, chunks[3]);
}
