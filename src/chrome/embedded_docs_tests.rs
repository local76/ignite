use super::*;
use crossterm::event::KeyCode;

#[test]
fn test_doc_for_f_key() {
    assert_eq!(doc_for_f_key(1), Some("README.md"));
    assert_eq!(doc_for_f_key(7), Some("CONTRIBUTING.md"));
    assert_eq!(doc_for_f_key(0), None);
    assert_eq!(doc_for_f_key(8), None);
}

#[test]
fn test_is_doc_f_key() {
    assert_eq!(is_doc_f_key(KeyCode::F(1)), Some("README.md"));
    assert_eq!(is_doc_f_key(KeyCode::F(7)), Some("CONTRIBUTING.md"));
    assert_eq!(is_doc_f_key(KeyCode::F(8)), None);
    assert_eq!(is_doc_f_key(KeyCode::Char('h')), None);
}

#[test]
fn test_open_embedded_markdown() {
    assert_eq!(open_embedded_markdown(KeyCode::F(3)), Some("LICENSE.md"));
    assert_eq!(open_embedded_markdown(KeyCode::Char('q')), None);
}

#[test]
fn test_doc() {
    assert_eq!(doc("README.md"), Some("README.md"));
    assert_eq!(doc("NOT_FOUND.md"), None);
}
