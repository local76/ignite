#![allow(dead_code)]
use crossterm::event::KeyCode;

/// Standard text entry box state and event handler for Ratatui TUIs.
#[derive(Debug, Clone, Default)]
pub struct TextBox {
    /// Current string content buffer.
    pub text: String,
    /// Index of the typing cursor within the string buffer.
    pub cursor_pos: usize,
    /// Whether this textbox is active and capturing character keystrokes.
    pub active: bool,
}

impl TextBox {
    pub fn new() -> Self {
        Self::default()
    }

    /// Process keystrokes to edit the buffer and move the cursor.
    pub fn handle_key(&mut self, code: KeyCode) {
        if !self.active {
            return;
        }
        match code {
            KeyCode::Char(c) => {
                self.text.insert(self.cursor_pos, c);
                self.cursor_pos += 1;
            }
            KeyCode::Backspace => {
                if self.cursor_pos > 0 {
                    self.cursor_pos -= 1;
                    self.text.remove(self.cursor_pos);
                }
            }
            KeyCode::Delete => {
                if self.cursor_pos < self.text.len() {
                    self.text.remove(self.cursor_pos);
                }
            }
            KeyCode::Left => {
                if self.cursor_pos > 0 {
                    self.cursor_pos -= 1;
                }
            }
            KeyCode::Right => {
                if self.cursor_pos < self.text.len() {
                    self.cursor_pos += 1;
                }
            }
            KeyCode::Home => {
                self.cursor_pos = 0;
            }
            KeyCode::End => {
                self.cursor_pos = self.text.len();
            }
            _ => {}
        }
    }

    /// Clear the text content buffer and reset cursor to index zero.
    pub fn clear(&mut self) {
        self.text.clear();
        self.cursor_pos = 0;
    }
}

