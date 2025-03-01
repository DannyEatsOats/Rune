use std::io;
use std::path::PathBuf;
use std::sync::Arc;
use devicons::Theme;
use tokio::sync::Mutex;

use crossterm::event::*;
use ratatui::{DefaultTerminal, widgets::*};

use crate::ui::{self, *};
use crate::manager::*;

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
            current_theme: 0,
        };

        app.themes = theme::Theme::init_themes();

        app
    }

    //Most code here will be changed, but it's a successful simulation, of what i want to do.
    
    /// Start the app. this is the main loop where *ui updates* and
    /// *events* get handled asyncronously
    pub fn run(terminal: &mut DefaultTerminal) -> io::Result<()> {
        let mut app = App::new();
        let mut i = 0;
        // Should be "!app.exit"
        while  i < 2 {
            // Later add the input blinker functionality here
            terminal.draw(|f| ui(f, &mut app))?;
            i+=1;

            if crossterm::event::poll(std::time::Duration::from_millis(50))? {
                if let Event::Key(key) = crossterm::event::read()? {
                    app.handle_key_event(key)?;
                }
            }
        }
        
        let items = Arc::clone(&app.items);
        tokio::spawn(async move {
            println!("{items:?}");
        });
        Ok(())
    }

    /// Handles a key related event from the user
    pub fn handle_key_event(&mut self, key_event: KeyEvent) -> io::Result<()> {
        if self.mode == AppMode::Normal && key_event.kind == KeyEventKind::Press {
            self.handle_normal_mode(key_event);
        } else if self.mode == AppMode::Search && key_event.kind == KeyEventKind::Press {
            self.handle_search_mode(key_event);
        }


        let mut i = 0;
        // I have to make the app ArcMutex in main and the clone it into the async task for later
        // use
        let items = Arc::clone(&self.items);
        tokio::spawn(async move {
            while i < 1000 {
                println!("Number: {i}");
                //items.lock().await.push(format!("|{i}|"));
                i += 1;
                tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
            }
        });
        Ok(())
    }

    pub fn handle_normal_mode(&mut self, key_event: KeyEvent) {

    }

    pub fn handle_search_mode(&mut self, key_event: KeyEvent) {

    }

    pub fn get_theme(&self) -> &theme::Theme {
        &self.themes[self.current_theme]
    }
}
