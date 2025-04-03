use std::io;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use crossterm::event::*;
use ratatui::{DefaultTerminal, widgets::*};

use crate::manager::*;
use crate::ui::*;

/// A struct representing the modes the app can be in.
#[derive(PartialEq, Eq)]
pub enum AppMode {
    Normal,
    Edit,
    Search,
    Compare,
}

/// A struct representing the App. It holds state and handles user events.
pub struct App {
    exit: bool,
    mode: AppMode,
    manager: Manager,
    items: Arc<Mutex<Vec<PathBuf>>>,
    themes: Vec<theme::Theme>,
    current_theme: usize,
    main_list_state: ListState,
    pub search_input: input::Input,
}

impl App {
    /// Creates an instance of *App*
    pub fn new() -> Self {
        let fm = Manager::new();
        let items = fm.get_current_dir().unwrap();
        let mut app = Self {
            exit: false,
            mode: AppMode::Normal,
            manager: fm,
            items: Arc::new(Mutex::new(items)),
            themes: Vec::new(),
            current_theme: 1,
            main_list_state: ListState::default(),
            search_input: input::Input::new(),
        };
        app.main_list_state.select(Some(0));
        app.themes = theme::Theme::init_themes();
        app.search_input.set_color(app.get_theme().get_fg());

        app
    }

    //Most code here will be changed, but it's a successful simulation, of what i want to do.

    /// Start the app. this is the main loop where *ui updates* and
    /// *events* get handled asyncronously
    pub fn run(terminal: &mut DefaultTerminal) -> io::Result<()> {
        let mut app = App::new();
        let mut ui = UI::new(&app);
        while !app.exit {
            // Later add the input blinker functionality here
            terminal.draw(|f| ui.draw(f, &mut app))?;
            app.correct_ml_state();

            if crossterm::event::poll(std::time::Duration::from_millis(50))? {
                if let Event::Key(key) = crossterm::event::read()? {
                    app.handle_key_event(&key)?;
                }
            }
        }

        //println!("{:?}", app.get_current_items().lock().unwrap());

        Ok(())
    }

    /// Handles a key related event from the user
    pub fn handle_key_event(&mut self, key_event: &KeyEvent) -> io::Result<()> {
        if self.mode == AppMode::Normal && key_event.kind == KeyEventKind::Press {
            self.handle_normal_mode(&key_event);
        } else if self.mode == AppMode::Search && key_event.kind == KeyEventKind::Press {
            self.handle_search_mode(key_event);
        }

        Ok(())
    }

    /// Handles normal mode keyevents, modifiers
    pub fn handle_normal_mode(&mut self, key_event: &KeyEvent) {
        match key_event.code {
            KeyCode::Char('q') => {
                self.exit = true;
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if let Some(selected) = self.main_list_state.selected() {
                    let next = (selected + 1).min(self.items.lock().unwrap().len());
                    self.main_list_state.select(Some(next));
                }
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if let Some(selected) = self.main_list_state.selected() {
                    let prev = selected.saturating_sub(1);
                    self.main_list_state.select(Some(prev));
                }
            }
            KeyCode::Enter | KeyCode::Char('l') => {
                if let Some(selected) = self.main_list_state.selected() {
                    if self.items.lock().unwrap().get(selected).is_none() {
                        return;
                    }
                    let new_path = self.items.lock().unwrap().get(selected).unwrap().clone();
                    self.change_dir(new_path);
                }
            }
            KeyCode::Backspace | KeyCode::Char('h') => {
                self.step_back();
            }
            KeyCode::Char('?') => {
                self.mode = AppMode::Search;
            }
            _ => {}
        }
    }

    /// Handles search mode keyevents, modifiers
    fn handle_search_mode(&mut self, key_event: &KeyEvent) {
        match key_event.modifiers {
            KeyModifiers::CONTROL => {
                if let KeyCode::Char('h') = key_event.code {
                    self.search_input.handle(input::InputType::DeletePrevWord);
                }
            }
            _ => self.handle_skey_code(key_event),
        }
    }

    /// Handles search mode keycodes (regular keys without modifiers)
    fn handle_skey_code(&mut self, key_event: &KeyEvent) {
        match key_event.code {
            KeyCode::Enter => {
                if !self.manager.is_searching() {
                    let items = Arc::clone(&self.items);
                    let term = self.search_input.get_value();
                    self.manager
                        .perform_search(term, items, self.main_list_state.selected().unwrap_or(0))
                        .unwrap();
                    self.mode = AppMode::Normal;
                }
            }
            KeyCode::Esc => {
                self.mode = AppMode::Normal;
            }
            KeyCode::Backspace => self.search_input.handle(input::InputType::DeleteChar),
            KeyCode::Char(c) => {
                self.search_input.handle(input::InputType::AppendChar(c));
            }
            _ => {}
        }
    }

    pub fn get_current_items(&self) -> Arc<Mutex<Vec<PathBuf>>> {
        Arc::clone(&self.items)
    }

    pub fn get_current_path(&self) -> &PathBuf {
        &self.manager.get_current_path()
    }

    pub fn get_theme(&self) -> &theme::Theme {
        &self.themes[self.current_theme]
    }

    pub fn get_ml_state(&mut self) -> &mut ListState {
        &mut self.main_list_state
    }

    pub fn get_mode(&self) -> &AppMode {
        &self.mode
    }

    pub fn step_back(&mut self) {
        //[[TODO]] I'll need better error handling here
        if let Ok(cursor_idx) = self.manager.step_back() {
            self.items = Arc::new(Mutex::new(self.manager.get_current_dir().unwrap()));
            let cursor_idx = if cursor_idx >= self.items.lock().unwrap().len() {
                0
            } else {
                cursor_idx
            };
            self.main_list_state.select(Some(cursor_idx));
        }
    }

    pub fn change_dir(&mut self, new_path: PathBuf) {
        if let Ok(items) = self.manager.change_dir(
            new_path.clone(),
            self.main_list_state.selected().unwrap_or(0),
        ) {
            self.items = Arc::new(Mutex::new(items));
        }
    }

    fn correct_ml_state(&mut self) {
        if self.items.lock().unwrap().is_empty() {
            self.main_list_state.select(Some(0));
        }
    }
}
