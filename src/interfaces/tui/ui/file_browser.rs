use ratatui::{
    Frame,
    layout::{Margin, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, List, ListItem},
};

use super::common::centered_rect;
use crate::interfaces::tui::app::App;

pub fn draw_file_browser_screen(frame: &mut Frame, app: &mut App, area: Rect) {
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
    let items: Vec<ListItem> = app
        .dir_entries
        .iter()
        .enumerate()
        .map(|(idx, path)| {
            let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("..");

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
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title("Select JSON file to import (Up/Down to navigate, Enter to select)")
            .border_style(Style::default().fg(Color::Yellow)),
    );

    frame.render_widget(list, inner_area);
}
