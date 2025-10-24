use ratatui::{
    Frame,
    layout::{Margin, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Paragraph},
};

use super::common::centered_rect;

pub fn draw_exiting_screen(frame: &mut Frame, area: Rect) {
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
        Line::from(vec![Span::styled(
            "Are you sure you want to exit?",
            Style::default().fg(Color::White).bold(),
        )]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Press [y] to quit, [n] to cancel",
            Style::default().fg(Color::DarkGray),
        )]),
    ];

    let paragraph = Paragraph::new(text).alignment(ratatui::layout::Alignment::Center);

    frame.render_widget(paragraph, inner_area);
}
