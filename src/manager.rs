use core::{fmt, time};
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::io::{BufRead, Read};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime};
use std::{fs, io, u32, usize};

use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

use crate::ui::UI;

#[derive(Debug)]
pub enum ManagerError {
    InvalidPath,
    NoPermission,
}

impl Error for ManagerError {}

impl fmt::Display for ManagerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

pub enum IndexOption {
    Simple,
    Recursive,
}

pub enum OpenOption {
    Full,
    Preview,
}

struct Flags {
    pub is_searching: Arc<Mutex<bool>>,
    pub is_indexing: Arc<Mutex<bool>>,
    pub is_loading: bool,
}

impl Flags {
    pub fn new() -> Self {
        Self {
            is_searching: Arc::new(Mutex::new(false)),
            is_indexing: Arc::new(Mutex::new(false)),
            is_loading: false,
        }
    }
}

struct Index {
    index: HashMap<String, HashSet<PathBuf>>,
    last_sync: Option<SystemTime>,
}

impl Index {
    pub fn new() -> Self {
        Self {
            index: HashMap::new(),
            last_sync: None,
        }
    }
}

pub struct Manager {
    root: PathBuf,
    homedir: PathBuf,
    current: PathBuf,
    flags: Flags,
    pathstack: Vec<(PathBuf, usize)>,
    index: Arc<Mutex<Index>>,
    cache: HashMap<String, HashSet<PathBuf>>,
}

impl Manager {
    /// Creates a new instace of the FileManager
    pub fn new() -> Self {
        let home = PathBuf::from(std::env::var("HOME").unwrap_or("/".to_string()));

        let mut manager = Self {
            root: PathBuf::from("/"),
            homedir: home.clone(),
            current: home.clone(),
            flags: Flags::new(),
            pathstack: Vec::new(),
            index: Arc::new(Mutex::new(Index::new())),
            cache: HashMap::new(),
        };

        let index = manager.index.lock().unwrap();

        if !PathBuf::from("index/index.json").exists() {
            drop(index);
            manager
                .build_index(&home, IndexOption::Recursive)
                .unwrap_or(());
        } else {
            drop(index);
            manager.flags.is_loading = true;
            let load_res = manager.load_index();

            //Might need a NONE check idk
            if load_res.is_err()
                || manager
                    .index
                    .lock()
                    .unwrap()
                    .last_sync
                    .unwrap()
                    .elapsed()
                    .unwrap_or(Duration::from_secs(0))
                    > Duration::from_secs(60 * 60 * 24 * 5)
            {
                manager
                    .build_index(&home, IndexOption::Recursive)
                    .unwrap_or(());
            }

            manager.flags.is_loading = false;
        }

        manager
    }

    pub fn shutdown(&self) {
        let index = Arc::clone(&self.index);
        Manager::save_index(index);
    }

    pub fn get_current_path(&self) -> &PathBuf {
        &self.current
    }

    /// Returns the directory currently opened in the manager
    pub fn read_dir(&self, path: &PathBuf, option: OpenOption) -> std::io::Result<Vec<PathBuf>> {
        let mut items = Vec::new();

        let size = match option {
            OpenOption::Full => usize::MAX,
            OpenOption::Preview => 100,
        };
        for entry in std::fs::read_dir(&path)?.take(size) {
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

    pub fn read_file(&self, path: &PathBuf) -> io::Result<String> {
        if !path.is_file() {
            return Err(io::ErrorKind::IsADirectory.into());
        }

        let mut file = fs::File::open(path)?;
        let mut buffer = vec![0; 16 * 1024];
        let bytes = file.read(&mut buffer)?;
        buffer.truncate(bytes);

        match std::str::from_utf8(&buffer) {
            Ok(text) => {
                let preview: String = text.lines().take(100).collect::<Vec<&str>>().join("\n");
                Ok(preview)
            }
            Err(_) => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Binary or non-UTF-8 file",
            )),
        }
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

        // TODO: split term into filename and extension
        *self.flags.is_searching.lock().unwrap() = true;
        self.cache_search();

        self.index_search(term, &items);

        self.fallback_search(term, &items);

        Ok(())
    }

    ///Searches the cache of the manager for previous searches in the session
    fn cache_search(&self) {}

    ///Searches the indexed files and directorioes of the manager
    fn index_search(&mut self, term: &str, items: &Arc<Mutex<Vec<PathBuf>>>) {
        if let Some(res) = self.index.lock().unwrap().index.get(term) {
            let mut items = items.lock().unwrap();

            res.iter().for_each(|item| {
                items.push(item.clone());
            });
            drop(items);
        }
    }

    ///performs a recursive, multithreadded search traversing from the current direcoty
    fn fallback_search(&self, term: &str, items: &Arc<Mutex<Vec<PathBuf>>>) {
        let is_searching_arc = Arc::clone(&self.flags.is_searching);
        let items = Arc::clone(&items);
        let term = term.to_string();
        let path = self.current.clone();
        tokio::spawn(async move {
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

        // Temporary fix, for huge searches
        if items.lock().unwrap().len() > 2000 {
            return Ok(());
        }

        let content: Vec<_> = fs::read_dir(path)?.filter_map(Result::ok).collect();
        content.par_iter().for_each(|item| {
            //IT WOULD BE BETTER TO ACCUIRE THE LOCK HERE THEN DROP IT
            let path = item.path();
            if let Some(name) = path.file_name() {
                let term = term.to_lowercase();
                if name.to_string_lossy().to_lowercase().contains(&term)
                    && !items.lock().unwrap().contains(&path)
                {
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
        *self.flags.is_searching.lock().unwrap()
    }

    pub fn is_indexing(&self) -> bool {
        *self.flags.is_indexing.lock().unwrap()
    }

    pub fn is_loading(&self) -> bool {
        self.flags.is_loading
    }

    pub fn step_back(&mut self) -> Result<usize, ManagerError> {
        if let Some((prev, cursor_idx)) = self.pathstack.pop() {
            self.current = prev;
            return Ok(cursor_idx);
        }
        if self.current.pop() {
            return Ok(0);
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
        let items = self.read_dir(&self.current, OpenOption::Full).unwrap();

        Ok(items)
    }

    /// Public function for building the index. Spawns a thread so the building can run in the
    /// background. Calls Manager::index_recursion
    pub fn build_index(&self, dir: &PathBuf, option: IndexOption) -> Result<(), ManagerError> {
        self.index.lock().unwrap().last_sync = Some(SystemTime::now());
        let index = Arc::clone(&self.index);
        let index2 = Arc::clone(&self.index);
        let is_indexing = Arc::clone(&self.flags.is_indexing);
        let dir = dir.clone();

        std::thread::spawn(move || {
            *is_indexing.lock().unwrap() = true;
            Manager::index_recursion(index, &dir, option);
            Manager::save_index(index2);
            *is_indexing.lock().unwrap() = false;
        });
        Ok(())
    }

    /// Builds the index for the manager. Simple -> directory provided. Recursive -> recursive from
    /// directory provided
    fn index_recursion(
        index: Arc<Mutex<Index>>,
        dir: &PathBuf,
        option: IndexOption,
    ) -> Result<(), ManagerError> {
        if index.lock().unwrap().index.len() > 10000 {
            return Ok(());
        }
        let entry_it = fs::read_dir(dir);
        if entry_it.is_err() {
            return Err(ManagerError::NoPermission);
        }
        let items: Vec<_> = entry_it
            .unwrap()
            .filter_map(Result::ok)
            .filter(|item| !item.file_name().to_string_lossy().starts_with("."))
            .collect();

        if items.len() > 500 {
            return Ok(());
        }

        items.par_iter().for_each(|item| {
            let path = item.path();
            if let Some(name) = path.file_stem() {
                index
                    .lock()
                    .unwrap()
                    .index
                    .entry(name.to_string_lossy().to_string())
                    .and_modify(|paths| {
                        paths.insert(path.to_path_buf());
                    })
                    .or_insert_with(|| {
                        let mut hs = HashSet::new();
                        hs.insert(path.to_path_buf());
                        hs
                    });

                if let IndexOption::Recursive = option {
                    if path.is_dir() {
                        Manager::index_recursion(index.clone(), &path, IndexOption::Recursive);
                    }
                }
            }
        });
        Ok(())
    }

    pub fn save_index(index: Arc<Mutex<Index>>) -> Result<(), Box<dyn Error>> {
        fs::create_dir_all("index/").unwrap();
        let file1 = fs::File::create("index/index.json")?;
        let file2 = fs::File::create("index/last_sync.json")?;
        let index = &index.lock().unwrap();

        serde_json::to_writer(file1, &index.index)?;
        serde_json::to_writer(file2, &index.last_sync.unwrap_or(SystemTime::now()))?;
        Ok(())
    }

    pub fn load_index(&mut self) -> Result<(), Box<dyn Error>> {
        let file = fs::read_to_string("index/index.json")?;
        let mut index: HashMap<String, HashSet<PathBuf>> = serde_json::from_str(&file)?;
        let file = fs::read_to_string("index/last_sync.json")?;
        let last_sync: SystemTime = serde_json::from_str(&file)?;

        index.iter_mut().for_each(|(_, value)| {
            value.retain(|path| {
                let metadata = path.metadata();
                let mut valid = true;
                if let Ok(metadata) = metadata {
                    let mod_time = metadata.modified().unwrap_or(SystemTime::now());
                    valid = mod_time
                        .elapsed()
                        .unwrap_or(Duration::from_secs(0))
                        .as_secs()
                        < Duration::from_secs(60 * 60 * 24 * 30).as_secs();
                }
                path.exists() && valid
            });
        });

        let mut idx_lock = self.index.lock().unwrap();
        idx_lock.index = index;
        idx_lock.last_sync = Some(last_sync);
        drop(idx_lock);

        Ok(())
    }
}
