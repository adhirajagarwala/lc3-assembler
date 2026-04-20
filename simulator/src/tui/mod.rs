pub mod app;
pub mod ui;

use std::{
    io,
    time::{Duration, Instant},
};

use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};

use app::App;

/// Target ~30 fps.
const FRAME_PERIOD: Duration = Duration::from_millis(33);

/// Run the TUI event loop until the user quits.
pub fn run(mut app: App) -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;

    let result = event_loop(&mut terminal, &mut app);

    // Always restore terminal, even on error.
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

fn event_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
) -> io::Result<()> {
    let mut last_frame = Instant::now();

    loop {
        // ── Advance simulation ────────────────────────────────────────────────
        app.tick();

        // ── Render ────────────────────────────────────────────────────────────
        terminal.draw(|f| ui::render(f, app))?;

        // ── Poll for events (non-blocking within frame budget) ────────────────
        let elapsed = last_frame.elapsed();
        let timeout = FRAME_PERIOD.saturating_sub(elapsed);
        last_frame = Instant::now();

        if event::poll(timeout)? {
            match event::read()? {
                Event::Key(key) => {
                    // Ctrl-C / Ctrl-Q always quit.
                    if key.modifiers.contains(KeyModifiers::CONTROL) {
                        match key.code {
                            KeyCode::Char('c') | KeyCode::Char('q') => break,
                            _ => {}
                        }
                    }
                    match key.code {
                        KeyCode::Char(ch) => app.handle_char(ch),
                        KeyCode::Enter => app.handle_enter(),
                        KeyCode::Backspace => app.handle_backspace(),
                        KeyCode::Esc => app.handle_escape(),
                        KeyCode::Up => app.scroll_up(),
                        KeyCode::Down => app.scroll_down(),
                        _ => {}
                    }
                }
                Event::Resize(_, _) => {} // ratatui handles this automatically
                _ => {}
            }
        }

        if app.should_quit {
            break;
        }
    }
    Ok(())
}
