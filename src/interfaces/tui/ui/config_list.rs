use ratatui::{
    Frame,
    layout::{Margin, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
};

use super::widgets::Popup;
use crate::interfaces::tui::app::{App, ConfigListItem};
use crate::interfaces::tui::constants::popup;

pub fn draw_config_list_screen(frame: &mut Frame, app: &mut App, area: Rect) {
    let config_count = app
        .system
        .config_list_items
        .iter()
        .filter(|item| matches!(item, ConfigListItem::Config { .. }))
        .count();
    let title = format!("Runtime Configuration ({} items)", config_count);

    let inner_area = Popup::new(&title, popup::CONFIG_LIST)
        .theme_color(Color::Yellow)
        .margin(Margin::new(1, 1))
        .render(frame, area);

    // Calculate viewport for scrolling
    let visible_height = inner_area.height.saturating_sub(2) as usize; // reserve for header/footer
    let selected = app.system.config_selected_index;

    // Calculate scroll offset to keep selected item visible
    let scroll_offset = if selected >= visible_height {
        selected - visible_height + 1
    } else {
        0
    };

    let mut lines: Vec<Line> = Vec::new();

    // Render visible items
    for (i, item) in app
        .system
        .config_list_items
        .iter()
        .enumerate()
        .skip(scroll_offset)
    {
        if lines.len() >= visible_height {
            break;
        }

        match item {
            ConfigListItem::Header(category) => {
                lines.push(Line::from(vec![Span::styled(
                    format!(" [{}]", category),
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                )]));
            }
            ConfigListItem::Config { index, key: _ } => {
                let is_selected = i == selected;
                if let Some(cfg) = app.system.configs.get(*index) {
                    let mut spans = Vec::new();

                    // Selection indicator
                    if is_selected {
                        spans.push(Span::styled(
                            " > ",
                            Style::default().fg(Color::Yellow).bold(),
                        ));
                    } else {
                        spans.push(Span::raw("   "));
                    }

                    // Key
                    let key_style = if is_selected {
                        Style::default().fg(Color::Black).bg(Color::Yellow).bold()
                    } else {
                        Style::default().fg(Color::Cyan)
                    };
                    spans.push(Span::styled(cfg.key.as_str(), key_style));

                    // Separator
                    let sep_style = if is_selected {
                        Style::default().fg(Color::Black).bg(Color::Yellow)
                    } else {
                        Style::default().fg(Color::DarkGray)
                    };
                    spans.push(Span::styled(" = ", sep_style));

                    // Value (redact sensitive)
                    let display_value = if cfg.sensitive {
                        "[REDACTED]".to_string()
                    } else if cfg.value.len() > 30 {
                        format!("{}...", &cfg.value[..27])
                    } else {
                        cfg.value.clone()
                    };

                    let val_style = if is_selected {
                        Style::default().fg(Color::Black).bg(Color::Yellow)
                    } else if cfg.sensitive {
                        Style::default().fg(Color::DarkGray)
                    } else {
                        Style::default().fg(Color::White)
                    };
                    spans.push(Span::styled(display_value, val_style));

                    // Tags
                    if cfg.sensitive {
                        spans.push(Span::raw(" "));
                        spans.push(Span::styled(
                            " sensitive ",
                            Style::default().fg(Color::Black).bg(Color::Yellow),
                        ));
                    }
                    if cfg.requires_restart {
                        spans.push(Span::raw(" "));
                        spans.push(Span::styled(
                            " restart ",
                            Style::default().fg(Color::White).bg(Color::Red),
                        ));
                    }
                    if !cfg.editable {
                        spans.push(Span::raw(" "));
                        spans.push(Span::styled(
                            " readonly ",
                            Style::default().fg(Color::White).bg(Color::DarkGray),
                        ));
                    }

                    lines.push(Line::from(spans));
                }
            }
        }
    }

    // Footer hint
    lines.push(Line::from(""));
    lines.push(Line::from(vec![Span::styled(
        " [j/k] Navigate  [e/Enter] Edit  [r] Reset  [Esc] Back",
        Style::default().fg(Color::DarkGray),
    )]));

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, inner_area);
}
