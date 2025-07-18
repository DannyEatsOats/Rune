use std::{
    fmt::Display,
    fs::Metadata,
    path::PathBuf,
    sync::{Arc, Mutex},
    time::Duration,
};

use ratatui::widgets::ListState;

use crate::{
    manager::{Manager, OpenOption},
    ui::{input, theme},
};

/// A struct representing the modes the app can be in.
#[derive(PartialEq, Eq, Clone, Copy)]
pub enum AppMode {
    Normal,
    Edit(EditAction),
    Search,
    Navigate,
    Compare,
    Theme,
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum EditAction {
    Create,
    Delete,
    Rename,
    Move,
    Copy,
}

impl Display for AppMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppMode::Normal => write!(f, "Normal"),
            AppMode::Edit(_) => write!(f, "Edit"),
            AppMode::Search => write!(f, "Search"),
            AppMode::Navigate => write!(f, "Navigate"),
            AppMode::Compare => write!(f, "Compare"),
            AppMode::Theme => write!(f, "Theme"),
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
    pub theme_list_state: ListState,
    pub search_input: input::Input,
    pub nav_input: input::Input,
    pub edit_input: input::Input,
    pub cursor: (Option<PathBuf>, Option<Metadata>),
}

impl AppProperties {
    pub fn new() -> Self {
        let fm = Manager::new();
        let items = fm
            .read_dir(fm.get_current_path(), OpenOption::Full)
            .unwrap();
        let mut cursor = (None, None);
        if let Some(path) = items.first() {
            let mut metadata: Option<Metadata> = None;
            if let Ok(md) = path.metadata() {
                metadata = Some(md);
            }
            cursor = (Some(path.clone()), metadata);
        }
        let mut props = Self {
            exit: false,
            mode: AppMode::Normal,
            manager: fm,
            items: Arc::new(Mutex::new(items)),
            themes: Vec::new(),
            current_theme: 1,
            main_list_state: ListState::default(),
            theme_list_state: ListState::default(),
            search_input: input::Input::new(),
            nav_input: input::Input::new(),
            edit_input: input::Input::new(),
            cursor,
        };
        props.main_list_state.select(Some(0));
        props.theme_list_state.select(Some(1));
        props.themes = theme::Theme::init_themes();
        props.search_input.set_color(props.get_theme().get_fg());
        props.nav_input.set_color(props.get_theme().get_fg());
        props.edit_input.set_color(props.get_theme().get_fg());

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

    pub fn get_themes(&self) -> &Vec<theme::Theme> {
        &self.themes
    }

    pub fn get_ml_state(&mut self) -> &mut ListState {
        &mut self.main_list_state
    }

    pub fn get_tl_state(&mut self) -> &mut ListState {
        &mut self.theme_list_state
    }

    pub fn get_mode(&self) -> &AppMode {
        &self.mode
    }
}
