use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Tabs, Wrap},
    Frame,
};

use super::app::{App, Mode, Tab};

pub fn render(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(f.area());

    render_tabs(f, app, chunks[0]);

    match app.tab {
        Tab::Services => render_services(f, app, chunks[1]),
        Tab::Environment => render_environment(f, app, chunks[1]),
    }

    render_status(f, app, chunks[2]);

    if app.mode == Mode::Search {
        render_search_box(f, app);
    } else if app.mode == Mode::Edit {
        render_edit_box(f, app);
    }
}

fn render_tabs(f: &mut Frame, app: &App, area: Rect) {
    let titles = vec!["Services", "Environment"];
    let tabs = Tabs::new(titles)
        .block(Block::default().borders(Borders::ALL).title("rtui - Service & Environment Manager"))
        .select(match app.tab {
            Tab::Services => 0,
            Tab::Environment => 1,
        })
        .style(Style::default().fg(Color::White))
        .highlight_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD));
    f.render_widget(tabs, area);
}

fn render_services(f: &mut Frame, app: &mut App, area: Rect) {
    let services = app.filtered_services();
    let items: Vec<ListItem> = services
        .iter()
        .map(|s| ListItem::new(s.as_str()))
        .collect();

    let title = if app.search_query.is_empty() {
        "Services (↑/↓ to navigate, r to refresh, / to search, e to edit)".to_string()
    } else {
        format!("Services (filtered: {} results)", services.len())
    };

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(title))
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");

    f.render_stateful_widget(list, area, &mut app.service_list_state);
}

fn render_environment(f: &mut Frame, app: &mut App, area: Rect) {
    let env_vars = app.filtered_env_vars();
    let items: Vec<ListItem> = env_vars
        .iter()
        .map(|(k, v)| {
            let content = vec![Line::from(vec![
                Span::styled(k, Style::default().fg(Color::Cyan)),
                Span::raw("="),
                Span::styled(v, Style::default().fg(Color::Green)),
            ])];
            ListItem::new(content)
        })
        .collect();

    let title = if app.search_query.is_empty() {
        "Environment Variables (↑/↓ to navigate, r to refresh, / to search, e to edit)".to_string()
    } else {
        format!("Environment Variables (filtered: {} results)", env_vars.len())
    };

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(title))
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");

    f.render_stateful_widget(list, area, &mut app.env_list_state);
}

fn render_status(f: &mut Frame, app: &App, area: Rect) {
    let status = Paragraph::new(app.status_message.as_str())
        .block(Block::default().borders(Borders::ALL).title("Status"))
        .style(Style::default().fg(Color::Gray));
    f.render_widget(status, area);
}

fn render_search_box(f: &mut Frame, app: &App) {
    let area = centered_rect(60, 20, f.area());
    let block = Block::default()
        .title("Search")
        .borders(Borders::ALL)
        .style(Style::default().bg(Color::Black));

    let text = format!("/{}", app.search_query);
    let paragraph = Paragraph::new(text)
        .block(block)
        .style(Style::default().fg(Color::Yellow));

    f.render_widget(Clear, area);
    f.render_widget(paragraph, area);
}

fn render_edit_box(f: &mut Frame, app: &App) {
    let area = centered_rect(60, 30, f.area());

    if let Some(idx) = app.env_list_state.selected() {
        let env_vars = app.filtered_env_vars();
        if let Some((key, _)) = env_vars.get(idx) {
            let block = Block::default()
                .title(format!("Edit: {}", key))
                .borders(Borders::ALL)
                .style(Style::default().bg(Color::Black));

            let text = app.edit_value.clone();
            let paragraph = Paragraph::new(text)
                .block(block)
                .wrap(Wrap { trim: false })
                .style(Style::default().fg(Color::Green));

            f.render_widget(Clear, area);
            f.render_widget(paragraph, area);
        }
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
