use ratatui::{
    Frame,
    layout::{Margin, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Paragraph},
};

use super::common::centered_rect;

pub fn draw_help_screen(frame: &mut Frame, area: Rect) {
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
        Line::from(vec![Span::styled(
            "NAVIGATION",
            Style::default().fg(Color::Yellow).bold(),
        )]),
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
        Line::from(vec![Span::styled(
            "ACTIONS",
            Style::default().fg(Color::Yellow).bold(),
        )]),
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
        Line::from(vec![Span::styled(
            "SEARCH & UTILITY",
            Style::default().fg(Color::Yellow).bold(),
        )]),
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
        Line::from(vec![Span::styled(
            "FORM EDITING",
            Style::default().fg(Color::Yellow).bold(),
        )]),
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
        Line::from(vec![Span::styled(
            "STATUS INDICATORS",
            Style::default().fg(Color::Yellow).bold(),
        )]),
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
        Line::from(vec![Span::styled(
            "Press any key to close",
            Style::default().fg(Color::DarkGray),
        )]),
    ];

    let help_para = Paragraph::new(help_text).alignment(ratatui::layout::Alignment::Left);
    frame.render_widget(help_para, inner_area);
}
