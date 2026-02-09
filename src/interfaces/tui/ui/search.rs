use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph},
};

use super::widgets::Popup;
use crate::interfaces::tui::app::App;
use crate::interfaces::tui::constants::popup;

pub fn draw_search_screen(frame: &mut Frame, app: &App, area: Rect) {
    let inner_area = Popup::new("Search Links", popup::SEARCH)
        .theme_color(Color::Cyan)
        .margin(Margin::new(2, 2))
        .render(frame, area);

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
