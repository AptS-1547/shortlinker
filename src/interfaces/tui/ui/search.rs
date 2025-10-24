use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Paragraph},
};

use super::common::centered_rect;
use crate::interfaces::tui::app::App;

pub fn draw_search_screen(frame: &mut Frame, app: &App, area: Rect) {
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
    let search_input = Paragraph::new(&*app.search_input).block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title(format!("Search ({} chars)", app.search_input.len()))
            .border_style(Style::default().fg(Color::Yellow).bold()),
    );
    frame.render_widget(search_input, chunks[0]);

    // Instructions
    let instructions = vec![
        Line::from(""),
        Line::from(vec![Span::styled(
            "Type to search in codes and URLs",
            Style::default().fg(Color::DarkGray),
        )]),
        Line::from(""),
        Line::from(vec![
            Span::styled("[Enter]", Style::default().fg(Color::Green).bold()),
            Span::styled(" Apply  ", Style::default().fg(Color::White)),
            Span::styled("[Esc]", Style::default().fg(Color::Red).bold()),
            Span::styled(" Cancel", Style::default().fg(Color::White)),
        ]),
    ];

    let inst_para = Paragraph::new(instructions).alignment(ratatui::layout::Alignment::Center);
    frame.render_widget(inst_para, chunks[1]);
}
