use super::*;
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

#[test]
fn test_is_quit_key() {
    assert!(is_quit_key(KeyCode::Char('q'), KeyModifiers::NONE));
    assert!(is_quit_key(KeyCode::Char('Q'), KeyModifiers::NONE));
    assert!(is_quit_key(KeyCode::Esc, KeyModifiers::NONE));
    assert!(is_quit_key(KeyCode::Char('c'), KeyModifiers::CONTROL));

    assert!(!is_quit_key(KeyCode::Char('c'), KeyModifiers::NONE));
    assert!(!is_quit_key(KeyCode::Char('a'), KeyModifiers::NONE));
}

#[test]
fn test_is_quit_key_event() {
    let q_event = KeyEvent {
        code: KeyCode::Char('q'),
        modifiers: KeyModifiers::NONE,
        kind: KeyEventKind::Press,
        state: crossterm::event::KeyEventState::empty(),
    };
    assert!(is_quit_key_event(&q_event));

    let q_release = KeyEvent {
        code: KeyCode::Char('q'),
        modifiers: KeyModifiers::NONE,
        kind: KeyEventKind::Release,
        state: crossterm::event::KeyEventState::empty(),
    };
    assert!(!is_quit_key_event(&q_release));
}

#[test]
fn test_is_help_toggle_key() {
    assert!(is_help_toggle_key(KeyCode::Char('h')));
    assert!(is_help_toggle_key(KeyCode::Char('H')));
    assert!(!is_help_toggle_key(KeyCode::Char('x')));
}

#[test]
fn test_scroll_for_key() {
    // viewport_h = 10, line_count = 50. max_scroll = 50 - (10 + 10) = 30.
    // Up / 'k' -> scroll.saturating_sub(1)
    assert_eq!(scroll_for_key(KeyCode::Up, 5, 50, 10), Some(4));
    assert_eq!(scroll_for_key(KeyCode::Char('k'), 0, 50, 10), Some(0));

    // Down / 'j' -> scroll + 1 limited to max_scroll (30)
    assert_eq!(scroll_for_key(KeyCode::Down, 5, 50, 10), Some(6));
    assert_eq!(scroll_for_key(KeyCode::Char('j'), 30, 50, 10), Some(30));

    // PageUp -> scroll.saturating_sub(10)
    assert_eq!(scroll_for_key(KeyCode::PageUp, 15, 50, 10), Some(5));

    // PageDown -> scroll + 10 limited to max_scroll (30)
    assert_eq!(scroll_for_key(KeyCode::PageDown, 15, 50, 10), Some(25));
    assert_eq!(scroll_for_key(KeyCode::PageDown, 28, 50, 10), Some(30));

    // Other key should return None
    assert_eq!(scroll_for_key(KeyCode::Char('x'), 5, 50, 10), None);
}
