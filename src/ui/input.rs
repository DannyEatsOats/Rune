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
    AutoComplete(PathBuf),
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
            InputType::AutoComplete(mut path) => self.auto_complete(&mut path).unwrap_or(()),
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

    fn auto_complete(&mut self, base_path: &mut PathBuf) -> std::io::Result<()> {
        let term = &self.value;
        //let mut items = Vec::new();
        let mut split = term.split("/");
        split.next();

        //Needs to read current dir from the manager, or term if it is not relative
        //Absolute -> starts with '/'
        //Relative -> chain term to current path, and filter paths starting with this
        //            then find longest common prefix, and add that to 'self.value'.
        //            if prefix len == self.value len. then add /. repeat
        base_path.push(term);
        for entry in std::fs::read_dir(&base_path)? {
            let entry = entry?;
            let path = entry.path();

            if !path.is_dir() {
                continue;
            }

            println!("{base_path:?}");
            let path = path.to_string_lossy().to_string();
            if path.starts_with(base_path) {
                items.push(name);
            }
        }

        //items.sort();

        Ok(())
    }
}
