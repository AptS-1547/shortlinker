//! Inline search bar component

use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph},
};

use crate::interfaces::tui::app::App;

/// Draw the inline search bar
pub fn draw_inline_search_bar(frame: &mut Frame, app: &App, area: Rect) {
    let search_text = vec![Line::from(vec![
        Span::styled(
            "/",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(&app.search_input, Style::default().fg(Color::White)),
        Span::styled(
            "_",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::RAPID_BLINK),
        ),
    ])];

    let result_count = if app.is_searching && !app.filtered_links.is_empty() {
        format!(" ({} matches)", app.filtered_links.len())
    } else if app.is_searching && app.filtered_links.is_empty() && !app.search_input.is_empty() {
        " (no matches)".to_string()
    } else {
        String::new()
    };

    let block = Block::default()
        .title(format!("Search{}", result_count))
        .title_style(Style::default().fg(Color::Cyan))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Yellow));

    let paragraph = Paragraph::new(search_text).block(block);

    frame.render_widget(paragraph, area);
}
