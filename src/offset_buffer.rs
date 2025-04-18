use std::{fmt::Display, usize};

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::buffer;

#[derive(Debug, PartialEq, Eq)]
pub struct OffsetBuffer {
    buffer: String,
}

impl OffsetBuffer {
    pub fn new() -> Self {
        Self {
            buffer: String::new(),
        }
    }

    pub fn buff_event(&mut self, key: &KeyEvent) {
        if let KeyCode::Char(c) = key.code {
            if c.is_numeric() {
                self.buffer.push(c);
                return;
            }
        }
    }

    pub fn get_offset(&mut self) -> usize {
        if self.buffer.is_empty() {
            return 1;
        }
        let offset: usize = self.buffer.parse().unwrap();
        self.buffer.clear();
        offset
    }
}
