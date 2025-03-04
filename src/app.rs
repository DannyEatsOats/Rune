use devicons::Theme;
use std::io;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use crossterm::event::*;
use ratatui::{DefaultTerminal, widgets::*};

use crate::manager::*;
use crate::ui::{self, *};

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
    manager: Manager,
    items: Arc<Mutex<Vec<PathBuf>>>,
    mode: AppMode,
    themes: Vec<theme::Theme>,
    current_theme: usize,
    main_list_state: ListState,
}

impl App {
    /// Creates an instance of *App*
    pub fn new() -> Self {
        let fm = Manager::new();
        let items = fm.get_current_dir().unwrap();
        let mut app = Self {
            exit: false,
            manager: fm,
            items: Arc::new(Mutex::new(items)),
            mode: AppMode::Normal,
            themes: Vec::new(),
            current_theme: 1,
            main_list_state: ListState::default(),
        };
        app.main_list_state.select(Some(0));
        app.themes = theme::Theme::init_themes();

        app
    }

    //Most code here will be changed, but it's a successful simulation, of what i want to do.

    /// Start the app. this is the main loop where *ui updates* and
    /// *events* get handled asyncronously
    pub fn run(terminal: &mut DefaultTerminal) -> io::Result<()> {
        let mut app = App::new();
        while !app.exit {
            // Later add the input blinker functionality here
            terminal.draw(|f| ui(f, &mut app))?;
            app.correct_ml_state();

            if crossterm::event::poll(std::time::Duration::from_millis(50))? {
                if let Event::Key(key) = crossterm::event::read()? {
                    app.handle_key_event(key)?;
                }
            }
        }

        //println!("{:?}", app.get_current_items().lock().unwrap());

        Ok(())
    }

    /// Handles a key related event from the user
    pub fn handle_key_event(&mut self, key_event: KeyEvent) -> io::Result<()> {
        if self.mode == AppMode::Normal && key_event.kind == KeyEventKind::Press {
            self.handle_normal_mode(key_event);
        } else if self.mode == AppMode::Search && key_event.kind == KeyEventKind::Press {
            self.handle_search_mode(key_event);
        }

        Ok(())
    }

    pub fn handle_normal_mode(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('q') => {
                self.exit = true;
            }
            KeyCode::Down => {
                if let Some(selected) = self.main_list_state.selected() {
                    let next = (selected + 1).min(self.items.lock().unwrap().len());
                    self.main_list_state.select(Some(next));
                }
            }
            KeyCode::Up => {
                if let Some(selected) = self.main_list_state.selected() {
                    let prev = selected.saturating_sub(1);
                    self.main_list_state.select(Some(prev));
                }
            }
            KeyCode::Enter => {
                if !self.manager.is_searching() {
                    let items = Arc::clone(&self.items);
                    self.manager.perform_search("fasz", items).unwrap();
                }
            }
            _ => {}
        }
    }

    pub fn handle_search_mode(&mut self, key_event: KeyEvent) {}

    pub fn get_current_items(&self) -> Arc<Mutex<Vec<PathBuf>>> {
        Arc::clone(&self.items)
    }

    pub fn get_theme(&self) -> &theme::Theme {
        &self.themes[self.current_theme]
    }

    pub fn get_ml_state(&mut self) -> &mut ListState {
        &mut self.main_list_state
    }

    fn correct_ml_state(&mut self) {
        if self.items.lock().unwrap().is_empty() {
            self.main_list_state.select(Some(0));
        }
    }
}
