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
            if last_char == b' ' || last_char == b'/' {
                break;
            }
        }
    }

    fn auto_complete(&mut self, base_path: &mut PathBuf) -> std::io::Result<()> {
        let term = self.value.clone();
        let mut items = Vec::new();
        let split: Vec<&str> = term.split("/").collect();

        for i in 0..split.len() - 1 {
            base_path.push(split[i]);
        }

        //Needs to read current dir from the manager, or term if it is not relative
        //Absolute -> starts with '/'

        for entry in std::fs::read_dir(&base_path)? {
            let entry = entry?;
            let path = entry.path();

            if !path.is_dir() {
                continue;
            }

            if path.file_name().is_some() {
                let name = path.file_name().unwrap().to_string_lossy().to_string();

                if name.starts_with(&split[split.len() - 1]) {
                    items.push(name);
                }
            }
        }

        if items.is_empty() {
            return Ok(());
        }

        if items.len() == 1 {
            let mut val = String::new();
            for i in 0..split.len() - 1 {
                val.push_str(split[i]);
                val.push_str("/");
            }
            val.push_str(&(items.remove(0) + "/"));
            self.value = val;

            return Ok(());
        }

        //Longest Common Prefix
        items.sort();
        let first = items.first().unwrap();
        let last = items.last().unwrap();
        let mut prefix = String::new();

        for (ch1, ch2) in first.chars().zip(last.chars()) {
            if ch1 == ch2 {
                prefix.push(ch1);
            } else {
                break;
            }
        }

        let mut val = String::new();
        for i in 0..split.len() - 1 {
            val.push_str(split[i]);
            val.push_str("/");
        }
        val.push_str(&prefix);
        self.value = val;

        Ok(())
    }
}
