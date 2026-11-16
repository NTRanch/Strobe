//! Thin binary: parse args, build the real facade, run.
//!
//! Full-screen ratatui where a TTY exists; a non-TTY line mode
//! (also forced with `--line`) shares the same execute path.

#![forbid(unsafe_code)]

use std::io::{BufRead, IsTerminal, Write};
use std::path::PathBuf;
use std::time::Duration;

use strobe_tui::app::{App, RunMode, Screen};
use strobe_tui::facade::{OpsFacade, RealFacade};
use strobe_tui::screens;

fn workspace_root() -> PathBuf {
    // The binary lives in the workspace; CARGO_MANIFEST_DIR's parent
    // chain is only valid at build time, so prefer the cwd when it
    // holds the workspace manifest, else the manifest dir.
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    if cwd.join("Cargo.toml").exists() && cwd.join("crates").exists() {
        return cwd;
    }
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or(cwd)
}

fn line_mode(app: &mut App, facade: &mut dyn OpsFacade) {
    let stdin = std::io::stdin();
    let mut stdout = std::io::stdout();
    for line in stdin.lock().lines() {
        let Ok(line) = line else { break };
        if line.trim().is_empty() {
            continue;
        }
        let response = app.handle_line(facade, line.trim());
        let _ = writeln!(stdout, "{response}");
        let _ = stdout.flush();
        if app.quit {
            break;
        }
    }
}

fn full_screen(app: &mut App, facade: &mut dyn OpsFacade) -> std::io::Result<()> {
    crossterm::terminal::enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    crossterm::execute!(stdout, crossterm::terminal::EnterAlternateScreen)?;
    let backend = ratatui::backend::CrosstermBackend::new(stdout);
    let mut terminal = ratatui::Terminal::new(backend)?;
    let result = event_loop(app, facade, &mut terminal);
    crossterm::terminal::disable_raw_mode()?;
    crossterm::execute!(
        terminal.backend_mut(),
        crossterm::terminal::LeaveAlternateScreen
    )?;
    result
}

fn event_loop(
    app: &mut App,
    facade: &mut dyn OpsFacade,
    terminal: &mut ratatui::Terminal<ratatui::backend::CrosstermBackend<std::io::Stdout>>,
) -> std::io::Result<()> {
    use crossterm::event::{self, Event, KeyCode};
    loop {
        terminal.draw(|frame| screens::render(frame, app, facade))?;
        if app.quit {
            return Ok(());
        }
        if !event::poll(Duration::from_millis(250))? {
            continue;
        }
        let Event::Key(key) = event::read()? else {
            continue;
        };
        match key.code {
            KeyCode::Char('q') => app.quit = true,
            KeyCode::Tab => {
                let idx = Screen::ALL
                    .iter()
                    .position(|s| *s == app.screen)
                    .unwrap_or(0);
                app.screen = Screen::ALL[(idx + 1) % Screen::ALL.len()];
            }
            KeyCode::Char('m') => {
                app.mode = match app.mode {
                    RunMode::Plan => RunMode::Auto,
                    RunMode::Auto => RunMode::Plan,
                };
            }
            KeyCode::Char('j') => app.op_idx = (app.op_idx + 1) % 3,
            KeyCode::Char('k') => app.op_idx = (app.op_idx + 2) % 3,
            KeyCode::Char('l') => app.target_idx = (app.target_idx + 1) % 5,
            KeyCode::Char('h') => app.target_idx = (app.target_idx + 4) % 5,
            KeyCode::Enter => app.run_current(facade),
            KeyCode::Char('a') => app.approve(facade),
            KeyCode::Char('x') => app.abort(),
            _ => {}
        }
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let force_line = args.iter().any(|a| a == "--line");
    let root = workspace_root();
    let mut facade = RealFacade::new(root);
    let mut app = App::default();
    if force_line || !std::io::stdin().is_terminal() {
        line_mode(&mut app, &mut facade);
    } else if let Err(err) = full_screen(&mut app, &mut facade) {
        eprintln!("strobe-tui: terminal error: {err}");
        std::process::exit(1);
    }
}
