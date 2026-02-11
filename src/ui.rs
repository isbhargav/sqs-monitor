use crate::app::App;
use chrono::{DateTime, Local};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
};

pub fn draw(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(10),   // Main content
            Constraint::Length(3), // Status bar
        ])
        .split(frame.area());

    draw_header(frame, chunks[0]);
    draw_main_content(frame, app, chunks[1]);
    draw_status_bar(frame, app, chunks[2]);
}

fn draw_header(frame: &mut Frame, area: Rect) {
    let header = Paragraph::new("SQS Queue Monitor")
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" SQS Monitor ")
                .title_style(Style::default().fg(Color::Yellow)),
        );
    frame.render_widget(header, area);
}

fn draw_main_content(frame: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(area);

    draw_queue_list(frame, app, chunks[0]);
    draw_queue_details(frame, app, chunks[1]);
}

fn draw_queue_list(frame: &mut Frame, app: &App, area: Rect) {
    let items: Vec<ListItem> = app
        .queues
        .iter()
        .enumerate()
        .map(|(idx, queue)| {
            let is_dlq = queue.name.ends_with("-dlq") || queue.name.ends_with("_dlq");
            let msg_count = queue.approximate_messages;

            let msg_color = match msg_count {
                0 => Color::Green,
                1..=100 => Color::Yellow,
                _ => Color::Red,
            };

            let style = if idx == app.selected_index {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else if is_dlq {
                Style::default().fg(Color::Magenta)
            } else {
                Style::default()
            };

            let content = vec![Line::from(vec![
                Span::styled(
                    if idx == app.selected_index {
                        "> "
                    } else {
                        "  "
                    },
                    style,
                ),
                Span::styled(format!("{:<30}", queue.name), style),
                Span::styled(format!("{:>6}", msg_count), Style::default().fg(msg_color)),
            ])];

            ListItem::new(content).style(style)
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Queues (↑/↓ to navigate) ")
                .title_style(Style::default().fg(Color::Yellow)),
        )
        .highlight_style(
            Style::default()
                .fg(Color::Black)
                .bg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        );

    let mut list_state = ListState::default();
    list_state.select(Some(app.selected_index));

    frame.render_stateful_widget(list, area, &mut list_state);
}

fn draw_queue_details(frame: &mut Frame, app: &App, area: Rect) {
    let content = if let Some(queue) = app.selected_queue() {
        let mut lines = vec![
            Line::from(vec![
                Span::styled(
                    "Queue Name: ",
                    Style::default().add_modifier(Modifier::BOLD),
                ),
                Span::raw(&queue.name),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("Messages:              ", Style::default().fg(Color::Cyan)),
                Span::raw(queue.approximate_messages.to_string()),
            ]),
            Line::from(vec![
                Span::styled("Messages In Flight:    ", Style::default().fg(Color::Cyan)),
                Span::raw(queue.approximate_messages_not_visible.to_string()),
            ]),
            Line::from(vec![
                Span::styled("Messages Delayed:      ", Style::default().fg(Color::Cyan)),
                Span::raw(queue.approximate_messages_delayed.to_string()),
            ]),
            Line::from(""),
        ];

        if let Some(details) = &app.selected_details {
            if let Some(arn) = &details.arn {
                lines.push(Line::from(vec![Span::styled(
                    "ARN: ",
                    Style::default().add_modifier(Modifier::BOLD),
                )]));
                lines.push(Line::from(vec![Span::raw(arn)]));
                lines.push(Line::from(""));
            }

            if let Some(retention) = details.message_retention_period {
                lines.push(Line::from(vec![
                    Span::styled("Retention Period:      ", Style::default().fg(Color::Cyan)),
                    Span::raw(format!("{} seconds", retention)),
                ]));
            }

            if let Some(timeout) = details.visibility_timeout {
                lines.push(Line::from(vec![
                    Span::styled("Visibility Timeout:    ", Style::default().fg(Color::Cyan)),
                    Span::raw(format!("{} seconds", timeout)),
                ]));
            }

            if let Some(max_size) = details.maximum_message_size {
                lines.push(Line::from(vec![
                    Span::styled("Max Message Size:      ", Style::default().fg(Color::Cyan)),
                    Span::raw(format!("{} bytes", max_size)),
                ]));
            }

            if let Some(delay) = details.delay_seconds {
                lines.push(Line::from(vec![
                    Span::styled("Delay Seconds:         ", Style::default().fg(Color::Cyan)),
                    Span::raw(delay.to_string()),
                ]));
            }

            if let Some(created) = details.created_timestamp {
                let dt = DateTime::from_timestamp(created, 0)
                    .map(|dt| dt.with_timezone(&Local))
                    .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                    .unwrap_or_else(|| "N/A".to_string());
                lines.push(Line::from(""));
                lines.push(Line::from(vec![
                    Span::styled("Created:               ", Style::default().fg(Color::Cyan)),
                    Span::raw(dt),
                ]));
            }
        }

        lines
    } else {
        vec![Line::from("No queue selected")]
    };

    let details = Paragraph::new(content).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Queue Details ")
            .title_style(Style::default().fg(Color::Yellow)),
    );

    frame.render_widget(details, area);
}

fn draw_status_bar(frame: &mut Frame, app: &App, area: Rect) {
    let last_refresh = app
        .last_refresh
        .map(|dt| {
            dt.with_timezone(&Local)
                .format("%Y-%m-%d %H:%M:%S")
                .to_string()
        })
        .unwrap_or_else(|| "Never".to_string());

    let filter_status = if app.filter_non_empty { "ON" } else { "OFF" };

    let status_text = if app.awaiting_purge_confirmation || app.purge_in_progress {
        // Show confirmation prompt or purge-in-progress message
        app.status_message.clone()
    } else {
        // Normal status
        format!(
            "{} | Last Refresh: {} | Filter: {} | [Q]uit [R]efresh [F]ilter [Shift+X]Purge [↑/↓]Navigate",
            app.status_message, last_refresh, filter_status
        )
    };

    let status_style = if app.awaiting_purge_confirmation || app.purge_in_progress {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::White)
    };

    let status = Paragraph::new(status_text)
        .style(status_style)
        .block(Block::default().borders(Borders::ALL));

    frame.render_widget(status, area);
}
