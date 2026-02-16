use std::io::{self};
use std::time::Duration;

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Frame, Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
};

use crate::finite_state::search::{Dictionary, SearchResult};

struct App {
    input: String,
    results: Vec<SearchResult>,
    dictionary: Dictionary,
    search_duration: Duration,
}

impl App {
    fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let dictionary = Dictionary::new("dict.fst")?;
        Ok(Self {
            input: String::new(),
            results: Vec::new(),
            dictionary,
            search_duration: Duration::default(),
        })
    }

    fn on_key(&mut self, c: char) {
        self.input.push(c);
        self.perform_search();
    }

    fn on_backspace(&mut self) {
        self.input.pop();
        self.perform_search();
    }

    fn perform_search(&mut self) {
        if self.input.trim().is_empty() {
            self.results.clear();
            self.search_duration = Duration::default();
            return;
        }

        match self.dictionary.search(self.input.trim()) {
            Ok((results, duration)) => {
                self.results = results;
                self.search_duration = duration;
            }
            Err(_) => {
                // TODO: Handle error properly
            }
        }
    }
}

pub fn run_tui() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(debug_assertions)]
    {
        println!(
            "\x1b[33mWarning: Running in DEBUG mode. Performance will be slow. Use --release for benchmarks.\x1b[0m"
        );
        std::thread::sleep(Duration::from_secs(2));
    }

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app
    let mut app = match App::new() {
        Ok(app) => app,
        Err(err) => {
            disable_raw_mode()?;
            execute!(
                terminal.backend_mut(),
                LeaveAlternateScreen,
                DisableMouseCapture
            )?;
            terminal.show_cursor()?;
            eprintln!("Failed to initialize app (is dict.fst built?): {}", err);
            return Err(err);
        }
    };

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
        println!("{:?}", err);
    }

    Ok(())
}

fn run_app(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, app: &mut App) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, app))?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Esc => return Ok(()),
                KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    return Ok(());
                }
                KeyCode::Char(c) => app.on_key(c),
                KeyCode::Backspace => app.on_backspace(),
                _ => {}
            }
        }
    }
}

fn ui(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(3), // Input box
                Constraint::Min(1),    // Results list
                Constraint::Length(3), // Stats
            ]
            .as_ref(),
        )
        .split(f.area());

    // Input
    let input = Paragraph::new(app.input.as_str())
        .style(Style::default().fg(Color::Yellow))
        .block(Block::default().borders(Borders::ALL).title("Search"));
    f.render_widget(input, chunks[0]);

    // Results
    let items: Vec<ListItem> = app
        .results
        .iter()
        .enumerate()
        .map(|(i, res)| {
            // Convert Vec<u8> to String (lossy)
            let word = String::from_utf8_lossy(&res.key);
            let content = format!("{}. {} (score: {})", i + 1, word, res.value);
            ListItem::new(Line::from(vec![Span::raw(content)]))
        })
        .collect();

    let results_list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Results"))
        .highlight_style(Style::default().add_modifier(Modifier::BOLD));

    f.render_widget(results_list, chunks[1]);

    // Stats
    let stats = Paragraph::new(format!(
        "Found {} results in {:?}",
        app.results.len(),
        app.search_duration
    ))
    .style(Style::default().fg(Color::Gray))
    .block(Block::default().borders(Borders::ALL).title("Stats"));
    f.render_widget(stats, chunks[2]);
}
