use std::{
    fmt::Display,
    path::PathBuf,
    sync::{Arc, Mutex},
};

use ratatui::widgets::ListState;

use crate::{
    manager::Manager,
    ui::{input, theme},
};

/// A struct representing the modes the app can be in.
#[derive(PartialEq, Eq)]
pub enum AppMode {
    Normal,
    Edit,
    Search,
    Compare,
}

impl Display for AppMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppMode::Normal => write!(f, "Normal"),
            AppMode::Edit => write!(f, "Edit"),
            AppMode::Search => write!(f, "Search"),
            AppMode::Compare => write!(f, "Compare"),
        }
    }
}

pub struct AppProperties {
    pub exit: bool,
    pub mode: AppMode,
    pub manager: Manager,
    pub items: Arc<Mutex<Vec<PathBuf>>>,
    pub themes: Vec<theme::Theme>,
    pub current_theme: usize,
    pub main_list_state: ListState,
    pub search_input: input::Input,
    pub cursor: Option<PathBuf>,
}

impl AppProperties {
    pub fn new() -> Self {
        let fm = Manager::new();
        let items = fm.get_current_dir().unwrap();
        let mut cursor = None;
        if let Some(path) = items.first() {
            cursor = Some(path.clone());
        }
        let mut props = Self {
            exit: false,
            mode: AppMode::Normal,
            manager: fm,
            items: Arc::new(Mutex::new(items)),
            themes: Vec::new(),
            current_theme: 1,
            main_list_state: ListState::default(),
            search_input: input::Input::new(),
            cursor,
        };
        props.main_list_state.select(Some(0));
        props.themes = theme::Theme::init_themes();
        props.search_input.set_color(props.get_theme().get_fg());

        props
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
}
