use core::{fmt, time};
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use std::{fs, io};

use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

use crate::ui::UI;

#[derive(Debug)]
pub enum ManagerError {
    InvalidPath,
}

impl Error for ManagerError {}

impl fmt::Display for ManagerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

pub struct Manager {
    root: PathBuf,
    homedir: PathBuf,
    current: PathBuf,
    is_searching: Arc<Mutex<bool>>,
    pathstack: Vec<(PathBuf, usize)>,
    index: HashMap<String, HashSet<PathBuf>>,
    cache: HashMap<String, HashSet<PathBuf>>,
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
            cache: HashMap::new(),
        }
    }

    pub fn get_current_path(&self) -> &PathBuf {
        &self.current
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

    /// Starts the search process. First calling cache_search(), index_search() then fallback_search()
    pub fn perform_search(
        &mut self,
        term: &str,
        items: Arc<Mutex<Vec<PathBuf>>>,
        cursor_idx: usize,
    ) -> io::Result<()> {
        let term = term.trim();
        if term.is_empty() || term.contains("..") {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Invalid search term",
            ));
        }

        //pathstack could maybe just store references
        self.pathstack.push((self.current.clone(), cursor_idx));
        items.lock().unwrap().clear();

        self.cache_search();

        self.index_search(term, &items);

        self.fallback_search(term, &items);

        Ok(())
    }

    ///Searches the cache of the manager for previous searches in the session
    fn cache_search(&self) {}

    ///Searches the indexed files and directorioes of the manager
    fn index_search(&mut self, term: &str, items: &Arc<Mutex<Vec<PathBuf>>>) {
        if let Some(res) = self.index.get(term) {
            let mut items = items.lock().unwrap();

            res.iter().for_each(|item| {
                items.push(item.clone());
            });
            drop(items);
        }
    }

    ///performs a recursive, multithreadded search traversing from the current direcoty
    fn fallback_search(&self, term: &str, items: &Arc<Mutex<Vec<PathBuf>>>) {
        let is_searching_arc = Arc::clone(&self.is_searching);
        let items = Arc::clone(&items);
        let term = term.to_string();
        let path = self.current.clone();
        tokio::spawn(async move {
            *is_searching_arc.lock().unwrap() = true;
            Manager::fallback_recursion(&term, path, items, Instant::now()).unwrap();
            *is_searching_arc.lock().unwrap() = false;
        });
    }

    ///Used as a helper function for fallback_search to implement recursion
    fn fallback_recursion(
        term: &str,
        path: PathBuf,
        items: Arc<Mutex<Vec<PathBuf>>>,
        delta_time: Instant,
    ) -> Result<(), Box<dyn Error>> {
        if delta_time.elapsed() > Duration::from_secs(20) {
            return Ok(());
        }

        if path.to_string_lossy().contains("/proc") {
            return Ok(());
        }

        if items.lock().unwrap().len() > 2000 {
            return Ok(());
        }

        let content: Vec<_> = fs::read_dir(path)?.filter_map(Result::ok).collect();
        content.par_iter().for_each(|item| {
            let path = item.path();
            if let Some(name) = path.file_name() {
                let term = term.to_lowercase();
                if name.to_string_lossy().to_lowercase().contains(&term) {
                    items.lock().unwrap().push(path.clone());
                    //BUILD PATH
                }
            }
            if path.is_dir() {
                let items = Arc::clone(&items);
                Manager::fallback_recursion(term, path, items, delta_time).unwrap_or(());
            }
        });

        Ok(())
    }

    pub fn is_searching(&self) -> bool {
        *self.is_searching.lock().unwrap()
    }

    pub fn step_back(&mut self) -> Result<usize, ManagerError> {
        if let Some((prev, cursor_idx)) = self.pathstack.pop() {
            self.current = prev;
            return Ok(cursor_idx);
        }
        Err(ManagerError::InvalidPath)
    }

    pub fn change_dir(
        &mut self,
        new_path: PathBuf,
        cursor_idx: usize,
    ) -> Result<Vec<PathBuf>, ManagerError> {
        if !new_path.exists() || !new_path.is_dir() {
            return Err(ManagerError::InvalidPath);
        }

        self.pathstack.push((self.current.clone(), cursor_idx));
        self.current = new_path;

        //I'll have to handle this error here better later on
        let items = self.get_current_dir().unwrap();

        Ok(items)
    }
}
