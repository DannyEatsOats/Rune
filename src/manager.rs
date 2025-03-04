use std::collections::{HashMap, HashSet};
use std::io;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Duration;

pub struct Manager {
    root: PathBuf,
    homedir: PathBuf,
    current: PathBuf,
    is_searching: Arc<Mutex<bool>>,
    pathstack: Vec<PathBuf>,
    index: HashMap<String, HashSet<PathBuf>>,
}

impl Manager {
    /// Creates a new instace of the FileManager
    pub fn new() -> Self {
        let home = PathBuf::from(std::env::var("HOME").unwrap_or("/".to_string()));

        Self {
            root: PathBuf::from("/"),
            homedir: home.clone(),
            current: home.clone(),
            is_searching: Arc::new(Mutex::new(false)),
            pathstack: Vec::new(),
            index: HashMap::new(),
        }
    }

    /// Returns the directory currently opened in the manager
    pub fn get_current_dir(&self) -> std::io::Result<Vec<PathBuf>> {
        let mut items = Vec::new();

        for entry in std::fs::read_dir(&self.current)? {
            let entry = entry?;
            let path = entry.path();

            items.push(path);
        }
        items.sort_by(|a, b| {
            // Helper function to categorize paths
            fn categorize(path: &PathBuf) -> (u8, String) {
                let is_hidden = path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .map(|name| name.starts_with('.'))
                    .unwrap_or(false);

                let is_folder = path.is_dir();

                let priority = if is_hidden {
                    2
                } else if is_folder {
                    0
                } else {
                    1
                };

                let name = path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or("")
                    .to_string();

                (priority, name)
            }

            let (priority_a, name_a) = categorize(a);
            let (priority_b, name_b) = categorize(b);

            priority_a
                .cmp(&priority_b)
                .then_with(|| name_a.cmp(&name_b))
        });

        Ok(items)
    }

    /// Starts the search process. First calling indexSearch() then fallbackSearch()
    pub fn perform_search(&self, term: &str, items: Arc<Mutex<Vec<PathBuf>>>) -> io::Result<()> {
        if term.is_empty() || term.contains("..") {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Invalid search term",
            ));
        }

        let term = PathBuf::from(term);
        let is_searching_arc = Arc::clone(&self.is_searching);

        items.lock().unwrap().clear();

        tokio::spawn(async move {
            *is_searching_arc.lock().unwrap() = true;
            for i in 0..=10 {
                items
                    .lock()
                    .unwrap()
                    .push(PathBuf::from(format!("{term:?}:{i}")));
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
            *is_searching_arc.lock().unwrap() = false;
        });

        Ok(())
    }

    pub fn is_searching(&self) -> bool {
        *self.is_searching.lock().unwrap()
    }
}
