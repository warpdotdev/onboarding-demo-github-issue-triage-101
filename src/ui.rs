use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame,
};

use crate::app::{App, InputMode};
use crate::github::Issue;

pub fn draw(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(0),    // Main content
            Constraint::Length(3), // Help bar
        ])
        .split(frame.area());

    draw_header(frame, app, chunks[0]);
    draw_main(frame, app, chunks[1]);
    draw_help(frame, app, chunks[2]);
}

fn draw_header(frame: &mut Frame, app: &App, area: Rect) {
    let filtered = app.filtered_issues();
    let title = if app.loading {
        format!(" {} - Loading...", app.repo)
    } else if let Some(err) = &app.error {
        format!(" {} - Error: {}", app.repo, err)
    } else {
        format!(" {} ({} issues)", app.repo, filtered.len())
    };

    let header = Paragraph::new(title)
        .style(Style::default().fg(Color::Cyan).bold())
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan)),
        );

    frame.render_widget(header, area);
}

fn draw_main(frame: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(area);

    draw_issue_list(frame, app, chunks[0]);
    draw_preview(frame, app, chunks[1]);
}

fn draw_issue_list(frame: &mut Frame, app: &App, area: Rect) {
    let filtered = app.filtered_issues();

    let items: Vec<ListItem> = filtered
        .iter()
        .enumerate()
        .map(|(i, issue)| {
            let is_selected = i == app.selected;
            issue_to_list_item(issue, is_selected)
        })
        .collect();

    let title = if app.filter.is_empty() {
        " Issues ".to_string()
    } else {
        format!(" Issues (filter: {}) ", app.filter)
    };

    let list = List::new(items).block(
        Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::White)),
    );

    frame.render_widget(list, area);
}

fn issue_to_list_item<'a>(issue: &Issue, selected: bool) -> ListItem<'a> {
    let mut style = Style::default();
    if selected {
        style = style.bg(Color::DarkGray).add_modifier(Modifier::BOLD);
    }

    // First line: issue number and title
    let title_line = Line::from(vec![
        Span::styled(
            format!("#{} ", issue.number),
            Style::default().fg(Color::Yellow),
        ),
        Span::styled(truncate(&issue.title, 35), style),
    ]);

    // Second line: labels
    let label_spans: Vec<Span> = issue
        .labels
        .iter()
        .take(3)
        .map(|l| {
            let color = parse_hex_color(&l.color).unwrap_or(Color::Gray);
            Span::styled(
                format!(" {} ", l.name),
                Style::default().bg(color).fg(Color::Black),
            )
        })
        .collect();

    let label_line = if label_spans.is_empty() {
        Line::from(Span::styled(
            "  no labels",
            Style::default().fg(Color::DarkGray),
        ))
    } else {
        Line::from(label_spans)
    };

    ListItem::new(vec![title_line, label_line, Line::from("")]).style(style)
}

fn draw_preview(frame: &mut Frame, app: &App, area: Rect) {
    let content = if let Some(issue) = app.selected_issue() {
        format_issue_preview(issue)
    } else {
        Text::from("No issue selected")
    };

    let preview = Paragraph::new(content).wrap(Wrap { trim: false }).block(
        Block::default()
            .title(" Preview ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::White)),
    );

    frame.render_widget(preview, area);
}

fn format_issue_preview(issue: &Issue) -> Text<'static> {
    let mut lines = vec![
        Line::from(vec![
            Span::styled(
                format!("#{} ", issue.number),
                Style::default().fg(Color::Yellow).bold(),
            ),
            Span::styled(issue.title.clone(), Style::default().bold()),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Author: ", Style::default().fg(Color::DarkGray)),
            Span::raw(issue.author.login.clone()),
            Span::styled("  Created: ", Style::default().fg(Color::DarkGray)),
            Span::raw(format_date(&issue.created_at)),
        ]),
    ];

    // Labels
    if !issue.labels.is_empty() {
        let label_text = issue
            .labels
            .iter()
            .map(|l| l.name.clone())
            .collect::<Vec<_>>()
            .join(", ");
        lines.push(Line::from(vec![
            Span::styled("Labels: ", Style::default().fg(Color::DarkGray)),
            Span::raw(label_text),
        ]));
    }

    lines.push(Line::from(""));
    lines.push(Line::styled(
        "─".repeat(40),
        Style::default().fg(Color::DarkGray),
    ));
    lines.push(Line::from(""));

    // Body
    if let Some(body) = &issue.body {
        for line in body.lines().take(20) {
            lines.push(Line::from(line.to_string()));
        }
        if body.lines().count() > 20 {
            lines.push(Line::styled(
                "... (truncated)",
                Style::default().fg(Color::DarkGray),
            ));
        }
    } else {
        lines.push(Line::styled(
            "No description",
            Style::default().fg(Color::DarkGray),
        ));
    }

    // Comments summary
    if !issue.comments.is_empty() {
        lines.push(Line::from(""));
        lines.push(Line::styled(
            "─".repeat(40),
            Style::default().fg(Color::DarkGray),
        ));
        lines.push(Line::styled(
            format!(" {} comments", issue.comments.len()),
            Style::default().fg(Color::Cyan),
        ));
    }

    Text::from(lines)
}

fn draw_help(frame: &mut Frame, app: &App, area: Rect) {
    let help_text = match app.input_mode {
        InputMode::Normal => {
            " j/↓: down  k/↑: up  Enter: open in browser  /: filter  Esc: clear filter  r: refresh  q: quit "
        }
        InputMode::Filter => " Type to filter  Enter/Esc: done  Backspace: delete ",
    };

    let help = Paragraph::new(help_text)
        .style(Style::default().fg(Color::DarkGray))
        .block(Block::default().borders(Borders::ALL));

    frame.render_widget(help, area);
}

// Helper functions

fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}…", &s[..max_len - 1])
    }
}

fn parse_hex_color(hex: &str) -> Option<Color> {
    let hex = hex.trim_start_matches('#');
    if hex.len() != 6 {
        return None;
    }
    let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
    let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
    let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
    Some(Color::Rgb(r, g, b))
}

fn format_date(iso: &str) -> String {
    // Simple: just take the date part
    iso.split('T').next().unwrap_or(iso).to_string()
}
