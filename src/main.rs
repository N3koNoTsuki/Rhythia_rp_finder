mod api;
mod models;

use std::io;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame, Terminal,
};

use api::RhythiaClient;
use models::Map;

// ── Sort ──────────────────────────────────────────────────────────────────────

#[derive(Clone, Copy, PartialEq)]
enum Sort {
    Plays,
    Date,
}

impl Sort {
    fn label(self) -> &'static str {
        match self {
            Sort::Plays => "Plays",
            Sort::Date => "Date",
        }
    }
    fn next(self) -> Self {
        match self {
            Sort::Plays => Sort::Date,
            Sort::Date => Sort::Plays,
        }
    }
}

// ── Focus ─────────────────────────────────────────────────────────────────────

#[derive(Clone, Copy, PartialEq)]
enum Focus {
    MinRp,
    MaxRp,
    Sort,
}

impl Focus {
    fn next(self) -> Self {
        match self {
            Focus::MinRp => Focus::MaxRp,
            Focus::MaxRp => Focus::Sort,
            Focus::Sort => Focus::MinRp,
        }
    }
    fn prev(self) -> Self {
        match self {
            Focus::MinRp => Focus::Sort,
            Focus::MaxRp => Focus::MinRp,
            Focus::Sort => Focus::MaxRp,
        }
    }
}

// ── Channel messages from fetch thread ────────────────────────────────────────

enum Msg {
    Progress(u64, u64),
    Done(Vec<Map>),
    Err(String),
}

// ── App state ─────────────────────────────────────────────────────────────────

struct App {
    loading: bool,
    load_fetched: u64,
    load_total: u64,
    load_error: Option<String>,

    maps: Vec<Map>,
    filtered: Vec<usize>,

    min_rp: String,
    max_rp: String,
    sort: Sort,
    focus: Focus,

    list_state: ListState,
    selected: usize,
}

impl App {
    fn new() -> Self {
        App {
            loading: true,
            load_fetched: 0,
            load_total: 0,
            load_error: None,
            maps: Vec::new(),
            filtered: Vec::new(),
            min_rp: String::new(),
            max_rp: String::new(),
            sort: Sort::Plays,
            focus: Focus::MinRp,
            list_state: ListState::default(),
            selected: 0,
        }
    }

    fn refilter(&mut self) {
        let low = self.min_rp.parse::<u64>().unwrap_or(0);
        let high = self.max_rp.parse::<u64>().unwrap_or(u64::MAX);

        let mut indices: Vec<usize> = self
            .maps
            .iter()
            .enumerate()
            .filter(|(_, m)| {
                let rp = m.max_rp();
                rp >= low && rp <= high
            })
            .map(|(i, _)| i)
            .collect();

        match self.sort {
            Sort::Plays => {
                indices.sort_by(|&a, &b| self.maps[b].play_count.cmp(&self.maps[a].play_count));
            }
            Sort::Date => {
                indices.sort_by(|&a, &b| self.maps[b].created_at.cmp(&self.maps[a].created_at));
            }
        }

        self.filtered = indices;

        if self.filtered.is_empty() {
            self.selected = 0;
            self.list_state.select(None);
        } else {
            self.selected = self.selected.min(self.filtered.len() - 1);
            self.list_state.select(Some(self.selected));
        }
    }

    fn handle_char(&mut self, c: char) {
        if !c.is_ascii_digit() {
            return;
        }
        match self.focus {
            Focus::MinRp => {
                self.min_rp.push(c);
                self.refilter();
            }
            Focus::MaxRp => {
                self.max_rp.push(c);
                self.refilter();
            }
            Focus::Sort => {}
        }
    }

    fn handle_backspace(&mut self) {
        match self.focus {
            Focus::MinRp => {
                self.min_rp.pop();
                self.refilter();
            }
            Focus::MaxRp => {
                self.max_rp.pop();
                self.refilter();
            }
            Focus::Sort => {}
        }
    }

    fn scroll_down(&mut self, step: usize) {
        if !self.filtered.is_empty() {
            self.selected = (self.selected + step).min(self.filtered.len() - 1);
            self.list_state.select(Some(self.selected));
        }
    }

    fn scroll_up(&mut self, step: usize) {
        self.selected = self.selected.saturating_sub(step);
        if !self.filtered.is_empty() {
            self.list_state.select(Some(self.selected));
        }
    }
}

// ── Rendering ─────────────────────────────────────────────────────────────────

fn render(f: &mut Frame, app: &mut App) {
    if app.loading || app.load_error.is_some() {
        render_loading(f, app);
    } else {
        render_main(f, app);
    }
}

fn render_loading(f: &mut Frame, app: &mut App) {
    let area = f.area();
    let block = Block::default()
        .title(" Rhythia RP Finder ")
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));
    let inner = block.inner(area);
    f.render_widget(block, area);

    if let Some(ref err) = app.load_error {
        let para = Paragraph::new(format!("\nErreur : {}\n\nAppuie sur q pour quitter.", err))
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::Red));
        f.render_widget(para, inner);
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Fill(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Fill(1),
        ])
        .split(inner);

    let pct = if app.load_total == 0 {
        0.0f64
    } else {
        app.load_fetched as f64 / app.load_total as f64
    };
    let bar_width = (chunks[2].width as usize).saturating_sub(14);
    let filled = ((pct * bar_width as f64) as usize).min(bar_width);
    let empty = bar_width - filled;

    let label = Paragraph::new("Chargement des maps ranked…")
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::Yellow));

    let bar = Paragraph::new(format!(
        "[{}{}]  {}/{}",
        "█".repeat(filled),
        "░".repeat(empty),
        app.load_fetched,
        app.load_total,
    ))
    .alignment(Alignment::Center)
    .style(Style::default().fg(Color::Green));

    f.render_widget(label, chunks[1]);
    f.render_widget(bar, chunks[2]);
}

fn render_main(f: &mut Frame, app: &mut App) {
    let area = f.area();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(1),
            Constraint::Fill(1),
            Constraint::Length(1),
        ])
        .split(area);

    render_filters(f, app, chunks[0]);
    render_count(f, app, chunks[1]);
    render_list(f, app, chunks[2]);
    render_help(f, chunks[3]);
}

fn render_filters(f: &mut Frame, app: &mut App, area: Rect) {
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(33),
            Constraint::Percentage(33),
            Constraint::Percentage(34),
        ])
        .split(area);

    for (i, (label, value, focus)) in [
        ("Min RP", app.min_rp.clone(), Focus::MinRp),
        ("Max RP", app.max_rp.clone(), Focus::MaxRp),
    ]
    .iter()
    .enumerate()
    {
        let focused = app.focus == *focus;
        let cursor = if focused { "▌" } else { "" };
        let display = format!("{}{}", value, cursor);
        let block = Block::default()
            .title(format!(" {label} "))
            .borders(Borders::ALL)
            .border_style(if focused {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default().fg(Color::DarkGray)
            });
        let para = Paragraph::new(display).block(block).style(if focused {
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        });
        f.render_widget(para, cols[i]);
    }

    let sort_focused = app.focus == Focus::Sort;
    let sort_block = Block::default()
        .title(" Tri ")
        .borders(Borders::ALL)
        .border_style(if sort_focused {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default().fg(Color::DarkGray)
        });
    let sort_para = Paragraph::new(format!("◀ {} ▶", app.sort.label()))
        .block(sort_block)
        .style(if sort_focused {
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        });
    f.render_widget(sort_para, cols[2]);
}

fn render_count(f: &mut Frame, app: &mut App, area: Rect) {
    let para = Paragraph::new(format!(
        " {} maps trouvées  (sur {} chargées)",
        app.filtered.len(),
        app.maps.len()
    ))
    .style(Style::default().fg(Color::Cyan));
    f.render_widget(para, area);
}

fn render_list(f: &mut Frame, app: &mut App, area: Rect) {
    let items: Vec<ListItem> = app
        .filtered
        .iter()
        .enumerate()
        .map(|(rank, &idx)| {
            let map = &app.maps[idx];
            let title = Line::from(vec![
                Span::styled(
                    format!("[#{}] ", rank + 1),
                    Style::default().fg(Color::DarkGray),
                ),
                Span::styled(
                    map.title.clone(),
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
            ]);
            let info = Line::from(vec![
                Span::raw("  "),
                Span::styled("Mapper: ", Style::default().fg(Color::DarkGray)),
                Span::styled(map.creator.clone(), Style::default().fg(Color::White)),
                Span::raw("  │  "),
                Span::styled(
                    format!("RP: {}", map.max_rp()),
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw("  │  ⭐ "),
                Span::styled(
                    format!("{:.2}", map.star_rating),
                    Style::default().fg(Color::White),
                ),
                Span::raw("  │  Plays: "),
                Span::styled(
                    format_number(map.play_count),
                    Style::default().fg(Color::White),
                ),
                Span::raw("  │  "),
                Span::raw(map.duration_str()),
            ]);
            let url = Line::from(vec![
                Span::raw("  "),
                Span::styled(map.url(), Style::default().fg(Color::Green)),
            ]);
            ListItem::new(vec![title, info, url, Line::raw("")])
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::TOP))
        .highlight_style(Style::default().bg(Color::DarkGray))
        .highlight_symbol("▶ ");

    f.render_stateful_widget(list, area, &mut app.list_state);
}

fn render_help(f: &mut Frame, area: Rect) {
    let para = Paragraph::new(
        " [Tab] champ suivant  [←/→] changer tri  [↑↓/jk] naviguer  [PgUp/Dn] ×10  [q] quitter",
    )
    .style(Style::default().fg(Color::DarkGray));
    f.render_widget(para, area);
}

// ── Helper ────────────────────────────────────────────────────────────────────

fn format_number(n: u64) -> String {
    let s = n.to_string();
    let mut out = String::new();
    for (i, c) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            out.push(' ');
        }
        out.push(c);
    }
    out.chars().rev().collect()
}

// ── Entry point ───────────────────────────────────────────────────────────────

fn main() -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let (tx, rx) = mpsc::channel::<Msg>();
    thread::spawn(move || {
        let client = match RhythiaClient::new() {
            Ok(c) => c,
            Err(e) => {
                let _ = tx.send(Msg::Err(e.to_string()));
                return;
            }
        };
        match client.fetch_all(|f, t| {
            let _ = tx.send(Msg::Progress(f, t));
        }) {
            Ok(maps) => {
                let _ = tx.send(Msg::Done(maps));
            }
            Err(e) => {
                let _ = tx.send(Msg::Err(e.to_string()));
            }
        }
    });

    let mut app = App::new();
    let result = run(&mut terminal, &mut app, rx);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

fn run<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
    rx: mpsc::Receiver<Msg>,
) -> Result<()> {
    loop {
        while let Ok(msg) = rx.try_recv() {
            match msg {
                Msg::Progress(f, t) => {
                    app.load_fetched = f;
                    app.load_total = t;
                }
                Msg::Done(maps) => {
                    app.maps = maps;
                    app.loading = false;
                    app.refilter();
                }
                Msg::Err(e) => {
                    app.load_error = Some(e);
                    app.loading = false;
                }
            }
        }

        terminal.draw(|f| render(f, app))?;

        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => break,
                    KeyCode::Esc => break,
                    _ if app.loading => {}
                    KeyCode::Tab => app.focus = app.focus.next(),
                    KeyCode::BackTab => app.focus = app.focus.prev(),
                    KeyCode::Left | KeyCode::Right if app.focus == Focus::Sort => {
                        app.sort = app.sort.next();
                        app.refilter();
                    }
                    KeyCode::Enter if app.focus == Focus::Sort => {
                        app.sort = app.sort.next();
                        app.refilter();
                    }
                    KeyCode::Up | KeyCode::Char('k') => app.scroll_up(1),
                    KeyCode::Down | KeyCode::Char('j') => app.scroll_down(1),
                    KeyCode::PageUp => app.scroll_up(10),
                    KeyCode::PageDown => app.scroll_down(10),
                    KeyCode::Char(c) => app.handle_char(c),
                    KeyCode::Backspace => app.handle_backspace(),
                    _ => {}
                }
            }
        }
    }
    Ok(())
}
