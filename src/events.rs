use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppEvent {
    Quit,
    Refresh,
    NextQueue,
    PreviousQueue,
    ToggleFilter,
    PurgeQueue,
    ConfirmPurge,
    CancelPurge,
}

pub fn poll_event(timeout: Duration) -> anyhow::Result<Option<AppEvent>> {
    if event::poll(timeout)? {
        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                return Ok(handle_key_event(key));
            }
        }
    }
    Ok(None)
}

fn handle_key_event(key: KeyEvent) -> Option<AppEvent> {
    match key.code {
        KeyCode::Char('q') | KeyCode::Esc => Some(AppEvent::Quit),
        KeyCode::Char('r') => Some(AppEvent::Refresh),
        KeyCode::Char('f') => Some(AppEvent::ToggleFilter),
        KeyCode::Down | KeyCode::Char('j') => Some(AppEvent::NextQueue),
        KeyCode::Up | KeyCode::Char('k') => Some(AppEvent::PreviousQueue),
        KeyCode::Char('X') => Some(AppEvent::PurgeQueue), // Shift+X
        KeyCode::Char('y') | KeyCode::Char('Y') => Some(AppEvent::ConfirmPurge),
        KeyCode::Char('n') | KeyCode::Char('N') => Some(AppEvent::CancelPurge),
        _ => None,
    }
}
