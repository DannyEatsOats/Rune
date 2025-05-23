use std::path::PathBuf;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::{io, thread};

use crossterm::event::*;
use ratatui::{DefaultTerminal, widgets::*};

use crate::app_properties::{AppMode, AppProperties};
use crate::manager::{self, *};
use crate::offset_buffer::{self, OffsetBuffer};
use crate::ui::*;

/// A struct representing the App. It holds state and handles user events.
pub struct App<'a> {
    properties: AppProperties,
    ui: UI<'a>,
    offset_buffer: OffsetBuffer,
}

impl<'a> App<'a> {
    /// Creates an instance of *App*
    pub fn new() -> Self {
        let mut properties = AppProperties::new();
        let mut app = Self {
            ui: UI::new(&properties),
            properties: properties,
            offset_buffer: OffsetBuffer::new(),
        };

        app
    }

    /// Start the app. this is the main loop where *ui updates* and
    /// *events* get handled asyncronously
    pub fn run(terminal: &mut DefaultTerminal) -> io::Result<()> {
        let mut app = App::new();
        while !app.properties.exit {
            terminal.draw(|f| app.ui.draw(f, &mut app.properties))?;
            app.correct_ml_state();

            if crossterm::event::poll(std::time::Duration::from_millis(50))? {
                if let Event::Key(key) = crossterm::event::read()? {
                    app.handle_key_event(&key)?;
                }
                //Makes sure cursor is set when searching, as there is no movement event to trigger
                //this
                if app.properties.manager.is_searching() {
                    app.generate_cursor(0);
                }
            }
        }

        Ok(())
    }

    /// Handles a key related event from the user
    pub fn handle_key_event(&mut self, key_event: &KeyEvent) -> io::Result<()> {
        if self.properties.mode == AppMode::Normal && key_event.kind == KeyEventKind::Press {
            self.handle_normal_mode(&key_event);
        } else if self.properties.mode == AppMode::Search && key_event.kind == KeyEventKind::Press {
            self.handle_search_mode(key_event);
        }

        Ok(())
    }

    /// Handles normal mode keyevents, modifiers
    pub fn handle_normal_mode(&mut self, key_event: &KeyEvent) {
        self.offset_buffer.buff_event(&key_event);
        match key_event.code {
            KeyCode::Char('q') => {
                self.properties.exit = true;
                self.properties.manager.shutdown();
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if let Some(selected) = self.properties.main_list_state.selected() {
                    let offset = self.offset_buffer.get_offset();
                    let next =
                        (selected + offset).min(self.properties.items.lock().unwrap().len() - 1);
                    self.generate_cursor(next);
                    self.properties.main_list_state.select(Some(next));
                }
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if let Some(selected) = self.properties.main_list_state.selected() {
                    let offset = self.offset_buffer.get_offset();
                    let prev = selected.saturating_sub(offset);
                    self.generate_cursor(prev);
                    self.properties.main_list_state.select(Some(prev));
                }
            }
            KeyCode::Enter | KeyCode::Char('l') => {
                if let Some(selected) = self.properties.main_list_state.selected() {
                    if self
                        .properties
                        .items
                        .lock()
                        .unwrap()
                        .get(selected)
                        .is_none()
                    {
                        return;
                    }
                    let new_path = self
                        .properties
                        .items
                        .lock()
                        .unwrap()
                        .get(selected)
                        .unwrap()
                        .clone();
                    self.change_dir(new_path);
                }
            }
            KeyCode::Backspace | KeyCode::Char('h') => {
                self.step_back();
            }
            KeyCode::Char('?') => {
                self.properties.mode = AppMode::Search;
            }
            _ => {}
        }
    }

    /// Handles search mode keyevents, modifiers
    fn handle_search_mode(&mut self, key_event: &KeyEvent) {
        match key_event.modifiers {
            KeyModifiers::CONTROL => {
                if let KeyCode::Char('h') = key_event.code {
                    self.properties
                        .search_input
                        .handle(input::InputType::DeletePrevWord);
                }
            }
            _ => self.handle_skey_code(key_event),
        }
    }

    /// Handles search mode keycodes (regular keys without modifiers)
    fn handle_skey_code(&mut self, key_event: &KeyEvent) {
        match key_event.code {
            KeyCode::Enter => {
                if !self.properties.manager.is_searching() {
                    let items = Arc::clone(&self.properties.items);
                    let term = self.properties.search_input.get_value();
                    // PANICS! on empty input
                    self.properties
                        .manager
                        .perform_search(
                            term,
                            items,
                            self.properties.main_list_state.selected().unwrap_or(0),
                        )
                        .unwrap();
                    self.properties.mode = AppMode::Normal;
                }
            }
            KeyCode::Esc => {
                self.properties.mode = AppMode::Normal;
            }
            KeyCode::Backspace => self
                .properties
                .search_input
                .handle(input::InputType::DeleteChar),
            KeyCode::Char(c) => {
                self.properties
                    .search_input
                    .handle(input::InputType::AppendChar(c));
            }
            _ => {}
        }
    }

    pub fn step_back(&mut self) {
        // TODO: I'll need better error handling here
        // TODO: I'll have to to create some kind of error buffer
        if let Ok(cursor_idx) = self.properties.manager.step_back() {
            self.properties.items = Arc::new(Mutex::new(
                self.properties
                    .manager
                    .read_dir(self.properties.get_current_path(), OpenOption::Full)
                    .unwrap(),
            ));
            let cursor_idx = if cursor_idx >= self.properties.items.lock().unwrap().len() {
                0
            } else {
                cursor_idx
            };
            self.properties.main_list_state.select(Some(cursor_idx));
            self.ui.set_main_items(&self.properties);

            self.generate_cursor(cursor_idx);
        }
    }

    pub fn change_dir(&mut self, new_path: PathBuf) {
        if !new_path.is_dir() {
            return;
        }
        if let Ok(items) = self.properties.manager.change_dir(
            new_path.clone(),
            self.properties.main_list_state.selected().unwrap_or(0),
        ) {
            self.properties.items = Arc::new(Mutex::new(items));
        }
        self.ui.set_main_items(&self.properties);
        self.generate_cursor(0);
    }

    fn generate_cursor(&mut self, idx: usize) {
        let path = self.properties.items.lock().unwrap().get(idx).cloned();
        let mut metadata = None;
        if let Some(p) = &path {
            if let Ok(md) = p.metadata() {
                metadata = Some(md);
            }
        }
        self.properties.cursor = (path, metadata);
    }

    fn correct_ml_state(&mut self) {
        if self.properties.items.lock().unwrap().is_empty() {
            self.properties.main_list_state.select(Some(0));
        }
    }
}
