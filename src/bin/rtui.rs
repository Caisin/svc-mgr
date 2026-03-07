use std::io;

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Tabs},
    Frame, Terminal,
};

use svc_mgr::{ServiceManager, TypedServiceManager};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Tab {
    Services,
    Environment,
}

struct App {
    tab: Tab,
    services: Vec<String>,
    env_vars: Vec<(String, String)>,
    service_list_state: ListState,
    env_list_state: ListState,
    status_message: String,
    should_quit: bool,
}

impl App {
    fn new() -> Self {
        let mut app = Self {
            tab: Tab::Services,
            services: Vec::new(),
            env_vars: Vec::new(),
            service_list_state: ListState::default(),
            env_list_state: ListState::default(),
            status_message: "Press 'q' to quit, Tab to switch tabs".to_string(),
            should_quit: false,
        };
        app.refresh_services();
        app.refresh_env_vars();
        app
    }

    fn refresh_services(&mut self) {
        match TypedServiceManager::native() {
            Ok(manager) => match manager.list() {
                Ok(action) => match action.exec() {
                    Ok(output) => match output.into_list() {
                        Ok(services) => {
                            self.services = services;
                            self.status_message = format!("Loaded {} services", self.services.len());
                        }
                        Err(e) => {
                            self.status_message = format!("Error parsing services: {}", e);
                        }
                    },
                    Err(e) => {
                        self.status_message = format!("Error executing list: {}", e);
                    }
                },
                Err(e) => {
                    self.status_message = format!("Error creating list action: {}", e);
                }
            },
            Err(e) => {
                self.status_message = format!("Error initializing manager: {}", e);
            }
        }
    }

    fn refresh_env_vars(&mut self) {
        let manager = svc_mgr::env::manager();
        match manager.list(svc_mgr::env::EnvScope::User) {
            Ok(vars) => {
                self.env_vars = vars.into_iter().collect();
                self.env_vars.sort_by(|a, b| a.0.cmp(&b.0));
                self.status_message = format!("Loaded {} environment variables", self.env_vars.len());
            }
            Err(e) => {
                self.status_message = format!("Error loading env vars: {}", e);
            }
        }
    }

    fn next_service(&mut self) {
        if self.services.is_empty() {
            return;
        }
        let i = match self.service_list_state.selected() {
            Some(i) => {
                if i >= self.services.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.service_list_state.select(Some(i));
    }

    fn previous_service(&mut self) {
        if self.services.is_empty() {
            return;
        }
        let i = match self.service_list_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.services.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.service_list_state.select(Some(i));
    }

    fn next_env(&mut self) {
        if self.env_vars.is_empty() {
            return;
        }
        let i = match self.env_list_state.selected() {
            Some(i) => {
                if i >= self.env_vars.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.env_list_state.select(Some(i));
    }

    fn previous_env(&mut self) {
        if self.env_vars.is_empty() {
            return;
        }
        let i = match self.env_list_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.env_vars.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.env_list_state.select(Some(i));
    }

    fn switch_tab(&mut self) {
        self.tab = match self.tab {
            Tab::Services => Tab::Environment,
            Tab::Environment => Tab::Services,
        };
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app and run
    let mut app = App::new();
    let res = run_app(&mut terminal, &mut app);

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("Error: {:?}", err);
    }

    Ok(())
}

fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, app))?;

        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                match key.code {
                    KeyCode::Char('q') => {
                        app.should_quit = true;
                    }
                    KeyCode::Tab => {
                        app.switch_tab();
                    }
                    KeyCode::Down | KeyCode::Char('j') => match app.tab {
                        Tab::Services => app.next_service(),
                        Tab::Environment => app.next_env(),
                    },
                    KeyCode::Up | KeyCode::Char('k') => match app.tab {
                        Tab::Services => app.previous_service(),
                        Tab::Environment => app.previous_env(),
                    },
                    KeyCode::Char('r') => match app.tab {
                        Tab::Services => app.refresh_services(),
                        Tab::Environment => app.refresh_env_vars(),
                    },
                    _ => {}
                }
            }
        }

        if app.should_quit {
            return Ok(());
        }
    }
}

fn ui(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(f.area());

    // Tabs
    let titles = vec!["Services", "Environment"];
    let tabs = Tabs::new(titles)
        .block(Block::default().borders(Borders::ALL).title("rtui - Service & Environment Manager"))
        .select(match app.tab {
            Tab::Services => 0,
            Tab::Environment => 1,
        })
        .style(Style::default().fg(Color::White))
        .highlight_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD));
    f.render_widget(tabs, chunks[0]);

    // Content
    match app.tab {
        Tab::Services => render_services(f, app, chunks[1]),
        Tab::Environment => render_environment(f, app, chunks[1]),
    }

    // Status bar
    let status = Paragraph::new(app.status_message.as_str())
        .block(Block::default().borders(Borders::ALL).title("Status"))
        .style(Style::default().fg(Color::Gray));
    f.render_widget(status, chunks[2]);
}

fn render_services(f: &mut Frame, app: &mut App, area: Rect) {
    let items: Vec<ListItem> = app
        .services
        .iter()
        .map(|s| ListItem::new(s.as_str()))
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Services (↑/↓ to navigate, r to refresh)"),
        )
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");

    f.render_stateful_widget(list, area, &mut app.service_list_state);
}

fn render_environment(f: &mut Frame, app: &mut App, area: Rect) {
    let items: Vec<ListItem> = app
        .env_vars
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

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Environment Variables (↑/↓ to navigate, r to refresh)"),
        )
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");

    f.render_stateful_widget(list, area, &mut app.env_list_state);
}
