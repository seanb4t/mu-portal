use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

/// What the client does with one key event.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Submit,
    Newline,
    SwitchFocus,
    Quit,
    Edit,
    Ignore,
}

/// Enter sends the pose. Shift+Enter, Alt+Enter and Ctrl+J insert a line
/// break, because a terminal without the kitty keyboard protocol sends
/// Shift+Enter as a plain Enter.
pub fn route(key: &KeyEvent) -> Action {
    if key.kind == KeyEventKind::Release {
        return Action::Ignore;
    }
    let m = key.modifiers;
    match key.code {
        KeyCode::Enter if m.intersects(KeyModifiers::SHIFT | KeyModifiers::ALT) => Action::Newline,
        KeyCode::Enter => Action::Submit,
        KeyCode::Char('j') if m == KeyModifiers::CONTROL => Action::Newline,
        KeyCode::Char('c') if m == KeyModifiers::CONTROL => Action::Quit,
        KeyCode::F(2) => Action::SwitchFocus,
        _ => Action::Edit,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::crossterm::event::KeyEventState;

    fn key(code: KeyCode, m: KeyModifiers) -> KeyEvent {
        KeyEvent::new(code, m)
    }

    #[test]
    fn enter_submits_and_modified_enter_breaks_the_line() {
        assert_eq!(
            route(&key(KeyCode::Enter, KeyModifiers::NONE)),
            Action::Submit
        );
        assert_eq!(
            route(&key(KeyCode::Enter, KeyModifiers::SHIFT)),
            Action::Newline
        );
        assert_eq!(
            route(&key(KeyCode::Enter, KeyModifiers::ALT)),
            Action::Newline
        );
        assert_eq!(
            route(&key(KeyCode::Char('j'), KeyModifiers::CONTROL)),
            Action::Newline
        );
    }

    #[test]
    fn a_release_does_nothing() {
        let release = KeyEvent {
            code: KeyCode::Enter,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Release,
            state: KeyEventState::NONE,
        };
        assert_eq!(route(&release), Action::Ignore);
    }

    #[test]
    fn plain_keys_go_to_the_editor() {
        assert_eq!(
            route(&key(KeyCode::Char('j'), KeyModifiers::NONE)),
            Action::Edit
        );
        assert_eq!(route(&key(KeyCode::Tab, KeyModifiers::NONE)), Action::Edit);
    }
}
