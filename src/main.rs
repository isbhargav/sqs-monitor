mod app;
mod aws;
mod events;
mod types;
mod ui;

use anyhow::Result;
use app::App;
use crossterm::{
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use events::{AppEvent, poll_event};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::io;
use std::time::{Duration, Instant};

#[tokio::main]
async fn main() -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app
    let mut app = App::new().await?;

    // Initial refresh
    app.refresh_queues().await?;

    // Main loop
    let mut last_auto_refresh = Instant::now();
    let result = run_app(&mut terminal, &mut app, &mut last_auto_refresh).await;

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

async fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
    last_auto_refresh: &mut Instant,
) -> Result<()> {
    loop {
        terminal.draw(|f| ui::draw(f, app))?;

        // Check for auto-refresh
        if last_auto_refresh.elapsed() >= app.refresh_interval {
            app.refresh_queues().await?;
            *last_auto_refresh = Instant::now();
        }

        // Poll for events with a short timeout
        if let Some(event) = poll_event(Duration::from_millis(100))? {
            match event {
                AppEvent::Quit => {
                    app.quit();
                    break;
                }
                AppEvent::Refresh => {
                    app.refresh_queues().await?;
                    *last_auto_refresh = Instant::now();
                }
                AppEvent::NextQueue => {
                    if !app.awaiting_purge_confirmation {
                        app.next_queue();
                        app.refresh_selected_details().await?;
                    }
                }
                AppEvent::PreviousQueue => {
                    if !app.awaiting_purge_confirmation {
                        app.previous_queue();
                        app.refresh_selected_details().await?;
                    }
                }
                AppEvent::ToggleFilter => {
                    if !app.awaiting_purge_confirmation {
                        app.toggle_filter();
                        app.refresh_selected_details().await?;
                    }
                }
                AppEvent::PurgeQueue => {
                    if !app.awaiting_purge_confirmation {
                        app.request_purge_confirmation();
                    }
                }
                AppEvent::ConfirmPurge => {
                    if app.awaiting_purge_confirmation {
                        if let Some((url, name)) = app.begin_purge() {
                            // Re-render to show "Purging..." before blocking on API call
                            terminal.draw(|f| ui::draw(f, app))?;
                            app.execute_purge(&url, &name).await?;
                        }
                        *last_auto_refresh = Instant::now();
                    }
                }
                AppEvent::CancelPurge => {
                    if app.awaiting_purge_confirmation {
                        app.cancel_purge();
                    }
                }
            }
        }

        if app.should_quit {
            break;
        }
    }

    Ok(())
}
