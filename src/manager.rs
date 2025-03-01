use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

pub struct Manager {
    root: PathBuf,
    homedir: PathBuf,
    current: PathBuf,
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
            pathstack: Vec::new(),
            index: HashMap::new(),
        }
    }

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
}
