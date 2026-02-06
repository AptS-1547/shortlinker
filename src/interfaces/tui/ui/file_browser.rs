use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, List, ListItem},
};

use super::widgets::Popup;
use crate::interfaces::tui::app::App;
use crate::interfaces::tui::constants::popup;

pub fn draw_file_browser_screen(frame: &mut Frame, app: &mut App, area: Rect) {
    let inner_area = Popup::new(
        &format!(" File Browser - {} ", app.current_dir.display()),
        popup::FILE_BROWSER,
    )
    .theme_color(Color::Cyan)
    .render(frame, area);

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
