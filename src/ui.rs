use crate::app::{App, CrateDetail, FeatureSelectState, Focus};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
};

pub fn ui(f: &mut Frame, app: &App) {
    let area = f.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(area);

    render_input(f, app, chunks[0]);
    render_center(f, app, chunks[1]);
    render_legend(f, app, chunks[2]);
}

fn render_input(f: &mut Frame, app: &App, area: Rect) {
    let focused = app.focus == Focus::Input;
    let accent = if focused {
        Color::Cyan
    } else {
        Color::DarkGray
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(accent))
        .title(Span::styled(
            " crate ",
            Style::default().fg(accent).add_modifier(Modifier::BOLD),
        ));

    f.render_widget(
        Paragraph::new(app.input.as_str())
            .block(block)
            .style(Style::default().fg(Color::White)),
        area,
    );

    if focused {
        f.set_cursor_position((area.x + 1 + app.cursor_pos as u16, area.y + 1));
    }
}

fn render_center(f: &mut Frame, app: &App, area: Rect) {
    if app.is_searching {
        f.render_widget(
            Paragraph::new(Span::styled(
                "Searching...",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::ITALIC),
            ))
            .block(center_block(" results ")),
            area,
        );
        return;
    }

    if app.results.is_empty() {
        let msg = if let Some(err) = &app.error {
            Span::styled(err.as_str(), Style::default().fg(Color::Red))
        } else {
            Span::styled(
                "Type a crate name and press Enter to search",
                Style::default().fg(Color::DarkGray),
            )
        };
        f.render_widget(Paragraph::new(msg).block(center_block(" results ")), area);
        return;
    }

    let panes = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(area);

    render_results(f, app, panes[0]);
    render_right_panel(f, app, panes[1]);
}

fn render_results(f: &mut Frame, app: &App, area: Rect) {
    let focused = matches!(app.focus, Focus::Results | Focus::FeatureSelect);
    let accent = if app.focus == Focus::Results {
        Color::Cyan
    } else {
        Color::DarkGray
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(accent))
        .title(Span::styled(
            " results ",
            Style::default().fg(Color::DarkGray),
        ));

    let items: Vec<ListItem> = app
        .results
        .iter()
        .map(|c| {
            ListItem::new(vec![
                Line::from(vec![
                    Span::styled(
                        c.name.clone(),
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        format!("  v{}", c.version),
                        Style::default().fg(Color::Yellow),
                    ),
                ]),
                Line::from(Span::styled(
                    format!("  {}", c.description),
                    Style::default().fg(Color::DarkGray),
                )),
            ])
        })
        .collect();

    let mut state = ListState::default();
    if focused {
        state.select(Some(app.selected));
    }

    f.render_stateful_widget(
        List::new(items)
            .block(block)
            .highlight_style(
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("▶ "),
        area,
        &mut state,
    );
}

fn render_right_panel(f: &mut Frame, app: &App, area: Rect) {
    if app.focus == Focus::FeatureSelect {
        if let Some(fs) = &app.feature_select {
            render_feature_select(f, fs, app.is_adding, area);
            return;
        }
    }

    if app.is_loading_detail && app.pending_feature_select {
        f.render_widget(
            Paragraph::new(Span::styled(
                "Loading features...",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::ITALIC),
            ))
            .block(info_block(" info ")),
            area,
        );
        return;
    }

    render_detail(f, app, area);
}

fn render_detail(f: &mut Frame, app: &App, area: Rect) {
    let block = info_block(" info ");

    if let Some(result) = &app.add_result {
        let paragraph = match result {
            Ok(msg) => Paragraph::new(Span::styled(
                format!("  {msg}"),
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            )),
            Err(e) => Paragraph::new(Span::styled(e.as_str(), Style::default().fg(Color::Red)))
                .wrap(Wrap { trim: false }),
        };
        f.render_widget(paragraph.block(block), area);
        return;
    }

    if app.is_loading_detail {
        f.render_widget(
            Paragraph::new(Span::styled(
                "Loading...",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::ITALIC),
            ))
            .block(block),
            area,
        );
        return;
    }

    match &app.detail {
        None => {
            f.render_widget(
                Paragraph::new(Span::styled(
                    "Press Enter to view crate info",
                    Style::default().fg(Color::DarkGray),
                ))
                .block(block),
                area,
            );
        }
        Some(Err(e)) => {
            f.render_widget(
                Paragraph::new(Span::styled(e.as_str(), Style::default().fg(Color::Red)))
                    .block(block)
                    .wrap(Wrap { trim: false }),
                area,
            );
        }
        Some(Ok(detail)) => {
            f.render_widget(detail_paragraph(detail, block), area);
        }
    }
}

fn detail_paragraph<'a>(detail: &'a CrateDetail, block: Block<'a>) -> Paragraph<'a> {
    let label = |s: &'a str| Span::styled(s, Style::default().fg(Color::DarkGray));

    let mut lines: Vec<Line> = vec![
        Line::from(vec![
            label("version:      "),
            Span::styled(
                detail.version.as_str(),
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            label("description:  "),
            Span::styled(
                detail.description.as_str(),
                Style::default().fg(Color::White),
            ),
        ]),
    ];

    if let Some(rv) = &detail.rust_version {
        lines.push(Line::from(vec![
            label("rust-version: "),
            Span::styled(rv.as_str(), Style::default().fg(Color::Green)),
        ]));
    }

    if let Some(repo) = &detail.repository {
        lines.push(Line::from(vec![
            label("repository:   "),
            Span::styled(
                repo.as_str(),
                Style::default()
                    .fg(Color::Blue)
                    .add_modifier(Modifier::UNDERLINED),
            ),
        ]));
    }

    if !detail.features.is_empty() {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "features:",
            Style::default()
                .fg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )));

        for feat in &detail.features {
            let (prefix, name_style) = if feat.enabled {
                (
                    "  + ",
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                )
            } else {
                ("    ", Style::default().fg(Color::Gray))
            };

            let mut spans = vec![Span::styled(format!("{prefix}{}", feat.name), name_style)];
            if !feat.deps.is_empty() {
                spans.push(Span::styled(
                    format!("  = {}", feat.deps),
                    Style::default().fg(Color::DarkGray),
                ));
            }
            lines.push(Line::from(spans));
        }
    }

    Paragraph::new(lines).block(block)
}

fn render_feature_select(f: &mut Frame, fs: &FeatureSelectState, is_adding: bool, area: Rect) {
    let title = if is_adding {
        " Adding... ".to_string()
    } else {
        format!(" add {} ", fs.crate_name)
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan))
        .title(Span::styled(
            title,
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ));

    let defaults_checkbox = if fs.no_default_features { "[ ]" } else { "[x]" };
    let defaults_style = if fs.no_default_features {
        Style::default().fg(Color::DarkGray)
    } else {
        Style::default().fg(Color::Green)
    };
    let defaults_row = ListItem::new(Line::from(vec![
        Span::styled(format!(" {defaults_checkbox} "), defaults_style),
        Span::styled("default features", defaults_style),
    ]));

    let feature_rows = fs.features.iter().map(|item| {
        let checkbox = if item.selected { "[x]" } else { "[ ]" };
        let style = if item.selected {
            Style::default().fg(Color::Green)
        } else {
            Style::default().fg(Color::Gray)
        };
        ListItem::new(Line::from(vec![
            Span::styled(format!(" {checkbox} "), style),
            Span::styled(item.name.clone(), style),
        ]))
    });

    let mut items: Vec<ListItem> = vec![defaults_row];
    items.extend(feature_rows);

    let mut state = ListState::default();
    if !is_adding {
        state.select(Some(fs.cursor));
    }

    f.render_stateful_widget(
        List::new(items)
            .block(block)
            .highlight_style(
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("▶ "),
        area,
        &mut state,
    );
}

fn render_legend(f: &mut Frame, app: &App, area: Rect) {
    let keys: &[(&str, &str)] = match app.focus {
        Focus::Input => &[("Enter", "search"), ("↓", "results"), ("Esc", "quit")],
        Focus::Results if app.in_rust_project => &[
            ("↑↓", "navigate"),
            ("Enter", "info"),
            ("a", "add to project"),
            ("Esc", "back"),
            ("^C", "quit"),
        ],
        Focus::Results => &[
            ("↑↓", "navigate"),
            ("Enter", "info"),
            ("Esc", "back"),
            ("^C", "quit"),
        ],
        Focus::FeatureSelect => &[
            ("↑↓", "navigate"),
            ("Space", "toggle"),
            ("Enter", "add"),
            ("Esc", "cancel"),
        ],
    };

    let mut spans: Vec<Span> = Vec::new();
    for (i, (key, desc)) in keys.iter().enumerate() {
        if i > 0 {
            spans.push(Span::raw("  "));
        }
        spans.push(Span::styled(
            format!(" {key} "),
            Style::default()
                .fg(Color::Black)
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        ));
        spans.push(Span::styled(
            format!(" {desc}"),
            Style::default().fg(Color::DarkGray),
        ));
    }

    f.render_widget(Paragraph::new(Line::from(spans)), area);
}

fn center_block(title: &str) -> Block<'_> {
    Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray))
        .title(Span::styled(title, Style::default().fg(Color::DarkGray)))
}

fn info_block(title: &str) -> Block<'_> {
    Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray))
        .title(Span::styled(title, Style::default().fg(Color::DarkGray)))
}
