use ratatui::style::Color;
use serde::de::value;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum InputType {
    AppendChar(char),
    DeleteChar,
    SetCursor(usize),
    DeletePrevWord,
    DeleteNextWord,
    GoToPrevWord,
    GoToNextWord,
}

#[derive(Debug)]
pub struct Input {
    value: String,
    color: Color,
}

impl Input {
    pub fn new() -> Self {
        Self {
            value: String::new(),
            color: Color::White,
        }
    }

    pub fn handle(&mut self, inp_type: InputType) {
        match inp_type {
            InputType::AppendChar(c) => self.append(c),
            InputType::DeleteChar => self.delete(),
            InputType::SetCursor(_) => todo!(),
            InputType::DeletePrevWord => self.dprev_word(),
            InputType::DeleteNextWord => todo!(),
            InputType::GoToPrevWord => todo!(),
            InputType::GoToNextWord => todo!(),
        }
    }

    pub fn get_value(&self) -> &String {
        &self.value
    }

    pub fn set_color(&mut self, color: Color) {
        self.color = color;
    }

    pub fn clear(&mut self) {
        self.value.clear();
    }

    fn append(&mut self, c: char) {
        self.value.push(c);
    }

    fn delete(&mut self) {
        self.value.pop();
    }

    fn dprev_word(&mut self) {
        while !self.value.is_empty() {
            let last_char = self.value.bytes().last().unwrap();
            self.value.pop();
            if last_char == b' ' {
                break;
            }
        }
    }
}
