use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph, Row, Table, TableState},
};

use crate::interfaces::tui::app::{App, SortColumn};
use crate::interfaces::tui::constants::URL_TRUNCATE_LENGTH;
use crate::interfaces::tui::ui::widgets::StatusIndicator;

/// Format a header cell with sort indicator
fn format_header(name: &str, col: SortColumn, app: &App) -> Span<'static> {
    if app.sort_column == Some(col) {
        let arrow = if app.sort_ascending { "▲" } else { "▼" };
        Span::styled(
            format!("{} {}", name, arrow),
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
    } else {
        Span::styled(
            name.to_string(),
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
    }
}

pub fn draw_main_screen(frame: &mut Frame, app: &mut App, area: Rect) {
    let total_on_page = app.display_count();

    if total_on_page == 0 {
        // Empty state or no search results
        let empty_text = if app.is_searching {
            vec![
                Line::from(""),
                Line::from(""),
                Line::from(vec![Span::styled(
                    "No links match your search",
                    Style::default()
                        .fg(Color::Gray)
                        .add_modifier(Modifier::BOLD),
                )]),
                Line::from(""),
                Line::from(vec![
                    Span::styled("Search query: ", Style::default().fg(Color::DarkGray)),
                    Span::styled(
                        app.search_query.as_deref().unwrap_or(&app.search_input),
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    ),
                ]),
                Line::from(""),
                Line::from(vec![
                    Span::styled("Press ", Style::default().fg(Color::DarkGray)),
                    Span::styled(
                        "[Esc]",
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(" to clear search", Style::default().fg(Color::DarkGray)),
                ]),
            ]
        } else {
            vec![
                Line::from(""),
                Line::from(""),
                Line::from(vec![Span::styled(
                    "No short links found",
                    Style::default()
                        .fg(Color::Gray)
                        .add_modifier(Modifier::BOLD),
                )]),
                Line::from(""),
                Line::from(vec![
                    Span::styled("Press ", Style::default().fg(Color::DarkGray)),
                    Span::styled(
                        "[a]",
                        Style::default()
                            .fg(Color::Green)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        " to create your first link",
                        Style::default().fg(Color::DarkGray),
                    ),
                ]),
            ]
        };

        let title = if app.is_searching {
            format!(
                "Search Results (\"{}\")",
                app.search_query.as_deref().unwrap_or(&app.search_input)
            )
        } else {
            "Short Links".to_string()
        };

        let empty = Paragraph::new(empty_text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(Color::DarkGray))
                    .title(title)
                    .title_style(Style::default().fg(Color::Cyan)),
            )
            .alignment(ratatui::layout::Alignment::Center);

        frame.render_widget(empty, area);
        return;
    }

    // 计算可见窗口（border 2行 + header 1行 + header margin 1行 = 4行开销）
    let visible_height = (area.height as usize).saturating_sub(4);
    app.last_visible_height = visible_height.max(1);

    // 确保 scroll_offset 合法
    let offset = app.scroll_offset.min(total_on_page.saturating_sub(1));
    app.scroll_offset = offset;
    let end = (offset + visible_height).min(total_on_page);

    // Table view for links with sort indicators
    let header = Row::new(vec![
        Span::raw("  "), // Selection indicator column
        format_header("Code", SortColumn::Code, app),
        format_header("URL", SortColumn::Url, app),
        format_header("Clicks", SortColumn::Clicks, app),
        format_header("Status", SortColumn::Status, app),
    ])
    .bottom_margin(1);

    // 虚拟渲染：只构建可见行的 Row
    let visible_links = &app.page_links[offset..end];
    let mut rows = Vec::with_capacity(end - offset);
    for link in visible_links {
        // Truncate URL if too long
        let display_url = if link.target.len() > URL_TRUNCATE_LENGTH {
            format!("{}...", &link.target[..URL_TRUNCATE_LENGTH])
        } else {
            link.target.clone()
        };

        // Use StatusIndicator to generate status text
        let indicator = StatusIndicator::new(link.password.is_some(), link.expires_at);
        let status_text = indicator.text();

        // Selection indicator
        let selection_prefix = if app.is_selected(&link.code) {
            Span::styled(
                "● ",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            )
        } else {
            Span::raw("  ")
        };

        let row = Row::new(vec![
            selection_prefix,
            Span::styled(
                link.code.clone(),
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(display_url, Style::default().fg(Color::Blue)),
            Span::styled(format!("{}", link.click), Style::default().fg(Color::Green)),
            Span::styled(status_text, Style::default().fg(Color::Yellow)),
        ]);

        rows.push(row);
    }

    // Build title with pagination, search and selection info
    let mut title_parts = vec![];

    let total_count = app.total_count;
    if app.is_searching {
        title_parts.push(format!(
            "Search: \"{}\" ({} found)",
            app.search_query.as_deref().unwrap_or(&app.search_input),
            total_count
        ));
    } else {
        title_parts.push(format!("Short Links ({})", total_count));
    }

    // 分页信息
    let total_pages = app.total_pages();
    if total_pages > 1 {
        title_parts.push(format!("Page {}/{}", app.current_page, total_pages));
    }

    if !app.selected_items.is_empty() {
        title_parts.push(format!("{} selected", app.selected_items.len()));
    }

    if let Some(col) = app.sort_column {
        let col_name = match col {
            SortColumn::Code => "Code",
            SortColumn::Url => "URL",
            SortColumn::Clicks => "Clicks",
            SortColumn::Status => "Status",
        };
        let dir = if app.sort_ascending { "↑" } else { "↓" };
        title_parts.push(format!("Sort: {}{}", col_name, dir));
    }

    let title = title_parts.join(" | ");

    let table = Table::new(
        rows,
        [
            ratatui::layout::Constraint::Length(2),  // Selection indicator
            ratatui::layout::Constraint::Length(15), // Code
            ratatui::layout::Constraint::Min(20),    // URL
            ratatui::layout::Constraint::Length(8),  // Clicks
            ratatui::layout::Constraint::Length(18), // Status
        ],
    )
    .header(header)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title(title)
            .title_style(
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
    )
    .row_highlight_style(Style::default().bg(Color::DarkGray).fg(Color::White))
    .highlight_symbol("▶ ")
    .column_spacing(1);

    // 虚拟 TableState：selected 调整为相对于可见窗口的偏移
    let mut virtual_state = TableState::default();
    if app.selected_index >= offset && app.selected_index < end {
        virtual_state.select(Some(app.selected_index - offset));
    }

    frame.render_stateful_widget(table, area, &mut virtual_state);
}
