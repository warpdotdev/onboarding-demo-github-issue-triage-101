mod app;
mod github;
mod ui;

use std::io;

use clap::Parser;
use crossterm::{
    event::{
        self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind, KeyModifiers,
    },
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};

use app::{App, InputMode};

#[derive(Parser)]
#[command(name = "issue-triage")]
#[command(about = "A TUI for triaging GitHub issues", long_about = None)]
struct Cli {
    /// Repository in owner/repo format (e.g., facebook/react)
    repo: String,

    /// Maximum number of issues to fetch
    #[arg(short, long, default_value = "100")]
    limit: u32,
}

fn main() -> io::Result<()> {
    let cli = Cli::parse();

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app and fetch issues
    let mut app = App::new(cli.repo);
    app.refresh();

    // Run the app
    let result = run_app(&mut terminal, &mut app);

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = result {
        eprintln!("Error: {err}");
    }

    Ok(())
}

fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui::draw(f, app))?;

        if let Event::Key(key) = event::read()? {
            if key.kind != KeyEventKind::Press {
                continue;
            }

            // Ctrl+C always exits
            if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
                return Ok(());
            }

            match app.input_mode {
                InputMode::Normal => match key.code {
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Char('j') | KeyCode::Down => app.next(),
                    KeyCode::Char('k') | KeyCode::Up => app.previous(),
                    KeyCode::Enter => app.open_selected(),
                    KeyCode::Char('/') => app.start_filter(),
                    KeyCode::Char('r') => app.refresh(),
                    KeyCode::Char('P') => {
                        let _ = app.copy_issue_prompt();
                    }
                    KeyCode::Esc => app.clear_filter(),
                    _ => {}
                },
                InputMode::Filter => match key.code {
                    KeyCode::Enter | KeyCode::Esc => app.exit_filter(),
                    KeyCode::Backspace => app.filter_pop(),
                    KeyCode::Char(c) => app.filter_push(c),
                    _ => {}
                },
            }
        }
    }
}
