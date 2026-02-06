use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::Paragraph,
};

use super::widgets::{InputField, Popup};
use crate::interfaces::tui::app::App;
use crate::interfaces::tui::constants::popup;

pub fn draw_export_filename_screen(frame: &mut Frame, app: &App, area: Rect) {
    let inner_area = Popup::new(" Export Links ", popup::EXPORT_FILENAME)
        .title_color(Color::Cyan)
        .border_color(Color::Green)
        .margin(Margin::new(2, 2))
        .render(frame, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2), // Instructions
            Constraint::Length(4), // Filename input + error space
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
    InputField::new("Filename", &app.export_filename_input)
        .active(true)
        .placeholder("e.g. my_links")
        .render(frame, chunks[1]);

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
