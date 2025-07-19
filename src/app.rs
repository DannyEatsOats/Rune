use std::path::PathBuf;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::{io, thread};

use crossterm::event::*;
use devicons::Theme;
use ratatui::{DefaultTerminal, widgets::*};

use crate::app_properties::{AppMode, AppProperties, EditAction};
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
        } else if self.properties.mode == AppMode::Navigate && key_event.kind == KeyEventKind::Press
        {
            self.handle_nav_mode(key_event);
        } else if self.properties.mode == AppMode::Theme && key_event.kind == KeyEventKind::Press {
            self.handle_theme_mode(key_event);
        } else if key_event.kind == KeyEventKind::Press {
            self.handle_edit_mode(key_event);
        }

        Ok(())
    }

    //TODO: optimization, always save current directory (properties.items) size, so you don't need
    //to borrow mutex lock

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
                    let next = (selected + offset).min(
                        self.properties
                            .items
                            .lock()
                            .unwrap()
                            .len()
                            .saturating_sub(1),
                    );
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

                    if new_path.is_dir() {
                        self.change_dir(new_path);
                    } else if new_path.is_file() {
                        _ = open::that_detached(new_path);
                    }
                }
            }
            KeyCode::Backspace | KeyCode::Char('h') => {
                self.step_back();
            }
            KeyCode::Char('?') => {
                self.properties.mode = AppMode::Search;
            }
            KeyCode::Char(':') => {
                self.properties.mode = AppMode::Navigate;
            }
            KeyCode::Char('a') => self.properties.mode = AppMode::Edit(EditAction::Create),
            KeyCode::Char('d') => self.properties.mode = AppMode::Edit(EditAction::Delete),
            KeyCode::Char('r') => self.properties.mode = AppMode::Edit(EditAction::Rename),
            KeyCode::Char('m') => self.properties.mode = AppMode::Edit(EditAction::Move),
            KeyCode::Char('c') => self.properties.mode = AppMode::Edit(EditAction::Copy),
            KeyCode::Char('t') => self.properties.mode = AppMode::Theme,
            _ => {}
        }
    }

    pub fn handle_theme_mode(&mut self, key_event: &KeyEvent) {
        match key_event.code {
            KeyCode::Down | KeyCode::Char('j') => {
                if let Some(selected) = self.properties.theme_list_state.selected() {
                    let next = (selected + 1).min(self.properties.themes.len().saturating_sub(1));
                    self.properties.theme_list_state.select(Some(next));
                    self.properties.current_theme = next;
                }
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if let Some(selected) = self.properties.theme_list_state.selected() {
                    let prev = selected.saturating_sub(1);
                    self.properties.theme_list_state.select(Some(prev));
                    self.properties.current_theme = prev;
                }
            }
            KeyCode::Enter
            | KeyCode::Char('l')
            | KeyCode::Char('q')
            | KeyCode::Esc
            | KeyCode::Char('h') => {
                if let Some(selected) = self.properties.theme_list_state.selected() {
                    if self.properties.themes.get(selected).is_none() {
                        return;
                    }
                    self.properties.current_theme = selected;
                    self.properties.mode = AppMode::Normal;
                    self.reload_dir();
                }
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
            _ => self.handle_searchkey_code(key_event),
        }
    }

    /// Handles navigation mode keyevents, modifiers
    fn handle_nav_mode(&mut self, key_event: &KeyEvent) {
        match key_event.modifiers {
            KeyModifiers::CONTROL => {
                if let KeyCode::Char('h') = key_event.code {
                    self.properties
                        .nav_input
                        .handle(input::InputType::DeletePrevWord);
                }
            }
            _ => self.handle_navkey_code(key_event),
        }
    }

    fn handle_edit_mode(&mut self, key_event: &KeyEvent) {
        match key_event.code {
            KeyCode::Esc => {
                self.properties.edit_input.clear();
                self.properties.mode = AppMode::Normal;
                return;
            }
            _ => {}
        }

        match &self.properties.mode {
            AppMode::Edit(x) => match x {
                EditAction::Create | EditAction::Move | EditAction::Copy | EditAction::Rename => {
                    match key_event.modifiers {
                        KeyModifiers::CONTROL => {
                            if let KeyCode::Char('h') = key_event.code {
                                self.properties
                                    .edit_input
                                    .handle(input::InputType::DeletePrevWord);
                            }
                        }
                        _ => self.handle_editkey_code(key_event, x.clone()),
                    }
                }
                EditAction::Delete => {
                    let idx = self.properties.main_list_state.selected();
                    if let Some(idx) = idx {
                        self.generate_cursor(idx);
                        if key_event.code == KeyCode::Enter {
                            if let (Some(path), _) = &self.properties.cursor {
                                _ = self.properties.manager.delete_fsitem(path);
                            }
                            self.properties.mode = AppMode::Normal;
                        }
                    }
                }
            },
            _ => {}
        }
        self.reload_dir();
    }

    fn reload_dir(&mut self) {
        self.change_dir(self.properties.manager.get_current_path().clone());
        self.properties.manager.after_reload();
    }

    fn handle_editkey_code(&mut self, key_event: &KeyEvent, action: EditAction) {
        match key_event.code {
            KeyCode::Enter => {
                let idx = self.properties.main_list_state.selected();
                if let None = idx {
                    return;
                }
                self.generate_cursor(idx.unwrap());
                match action {
                    EditAction::Create => {
                        let new_item = self.properties.edit_input.get_value();
                        _ = self.properties.manager.create_fsitem(&new_item);
                    }
                    EditAction::Rename => {
                        if let (Some(path), _) = &self.properties.cursor {
                            let source = path.clone();
                            let dest = self.properties.edit_input.get_value();
                            _ = self.properties.manager.rename_fsitem(source, dest);
                        }
                    }
                    EditAction::Move | EditAction::Copy => {
                        if let (Some(path), _) = &self.properties.cursor {
                            let source = path.clone();
                            let mut dest = PathBuf::from(self.properties.edit_input.get_value());

                            if !dest.exists() {
                                let mut val = self.properties.get_current_path().clone();
                                val.push(dest);
                                dest = val;
                            }

                            let mov_option = if action == EditAction::Move {
                                MoveOption::Move
                            } else {
                                MoveOption::Copy
                            };

                            _ = self
                                .properties
                                .manager
                                .move_fsitem(source, dest, mov_option);
                        }
                    }
                    _ => {}
                }
                //Maybe i could implement jump to item here
                self.properties.edit_input.clear();
                self.properties.mode = AppMode::Normal;
            }
            KeyCode::Esc => {
                self.properties.mode = AppMode::Normal;
            }
            KeyCode::Backspace => self
                .properties
                .edit_input
                .handle(input::InputType::DeleteChar),
            KeyCode::Tab => self
                .properties
                .edit_input
                .handle(input::InputType::AutoComplete(
                    self.properties.get_current_path().clone(),
                )),
            KeyCode::Char(c) => {
                self.properties
                    .edit_input
                    .handle(input::InputType::AppendChar(c));
            }
            _ => {}
        }
    }

    /// Handles search mode keycodes (regular keys without modifiers)
    fn handle_searchkey_code(&mut self, key_event: &KeyEvent) {
        match key_event.code {
            KeyCode::Enter => {
                if !self.properties.manager.is_searching() {
                    let items = Arc::clone(&self.properties.items);
                    let term = self.properties.search_input.get_value();
                    if term.is_empty() {
                        return;
                    }
                    // PANICS! on empty input
                    self.properties
                        .manager
                        .perform_search(
                            term,
                            items,
                            self.properties.main_list_state.selected().unwrap_or(0),
                        )
                        .unwrap();
                    self.properties.search_input.clear();
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

    /// Handles navigation mode keycodes (regular keys without modifiers)
    fn handle_navkey_code(&mut self, key_event: &KeyEvent) {
        match key_event.code {
            KeyCode::Enter => {
                if !self.properties.manager.is_searching() {
                    let term = self.properties.nav_input.get_value();
                    let mut path = PathBuf::from(term);
                    if !path.exists() {
                        let mut val = self.properties.get_current_path().clone();
                        val.push(path);
                        path = val;
                    }
                    self.change_dir(path);
                    self.properties.nav_input.clear();
                    self.properties.mode = AppMode::Normal;
                }
            }
            KeyCode::Esc => {
                self.properties.mode = AppMode::Normal;
            }
            KeyCode::Backspace => self
                .properties
                .nav_input
                .handle(input::InputType::DeleteChar),
            KeyCode::Tab => self
                .properties
                .nav_input
                .handle(input::InputType::AutoComplete(
                    self.properties.get_current_path().clone(),
                )),
            KeyCode::Char(c) => {
                self.properties
                    .nav_input
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
