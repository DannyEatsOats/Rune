use std::{
    error::Error,
    path::{Path, PathBuf},
};

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
    AutoComplete,
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
            InputType::AutoComplete => self.auto_complete().unwrap_or(()),
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

    fn auto_complete(&mut self) -> std::io::Result<()> {
        let term = PathBuf::from(&self.value);
        let mut items = Vec::new();

        //Needs to read current dir from the manager, or term if it is not relative
        for entry in std::fs::read_dir(&term)? {
            let entry = entry?;
            let path = entry.path();

            if !path.is_dir() {
                continue;
            }

            if path.file_name().is_some() {
                let name = path.file_name().unwrap().to_string_lossy().to_string();
                items.push(name);
            }
        }

        items.sort();

        Ok(())
    }
}
