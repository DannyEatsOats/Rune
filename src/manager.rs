use core::{fmt, time};
use std::collections::{HashMap, HashSet};
use std::env::vars;
use std::error::Error;
use std::fs::OpenOptions;
use std::io::{BufRead, Read, Write};
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

pub enum MoveOption {
    Move,
    Copy,
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

    pub fn after_reload(&mut self) {
        self.pathstack.pop();
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
        let split: Vec<&str> = term.split(".").collect();
        let mut filename = String::from(split[0]);
        for i in 1..split.len() - 1 {
            filename.push_str(&format!(".{}", split[i]));
        }
        let extension = if split.len() > 1 {
            Some(split.last().unwrap())
        } else {
            None
        };

        if let Some(res) = self.index.lock().unwrap().index.get(&filename) {
            let mut items = items.lock().unwrap();

            res.iter().for_each(|item| {
                if extension.is_none()
                    || (extension.is_some()
                        && item.extension().is_some()
                        && extension.unwrap() == &item.extension().unwrap().to_str().unwrap())
                {
                    items.push(item.clone());
                }
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
        let search_flag = Arc::clone(&self.flags.is_searching);
        tokio::spawn(async move {
            Manager::fallback_recursion(&term, path, items, search_flag, Instant::now()).unwrap();
            *is_searching_arc.lock().unwrap() = false;
        });
    }

    ///Used as a helper function for fallback_search to implement recursion
    fn fallback_recursion(
        term: &str,
        path: PathBuf,
        items: Arc<Mutex<Vec<PathBuf>>>,
        is_searching: Arc<Mutex<bool>>,
        delta_time: Instant,
    ) -> Result<(), Box<dyn Error>> {
        if delta_time.elapsed() > Duration::from_secs(20) {
            return Ok(());
        }

        if !*is_searching.lock().unwrap() {
            return Ok(());
        }

        if path.to_string_lossy().contains("/proc") {
            return Ok(());
        }

        if path.to_string_lossy().contains("/snap") {
            return Ok(());
        }

        if path.to_string_lossy().contains("state/nvim") {
            return Ok(());
        }

        // Temporary fix, for huge searches
        if items.lock().unwrap().len() > 2000 {
            return Ok(());
        }

        let content: Vec<_> = fs::read_dir(path)?.filter_map(Result::ok).collect();
        content.par_iter().for_each(|item| {
            let path = item.path();

            if let Some(name) = path.file_name() {
                let term = term.to_lowercase();
                let mut items_lock = items.lock().unwrap();
                if name.to_string_lossy().to_lowercase().contains(&term)
                    && !items_lock.contains(&path)
                {
                    if name.to_string_lossy().contains("titkos") {
                        let mut file = OpenOptions::new()
                            .create(true) // create if it doesn't exist
                            .append(true) // append to the file
                            .open("output.txt")
                            .unwrap();

                        writeln!(file, "{}", path.to_string_lossy()).unwrap();
                    }
                    items_lock.push(path.clone());
                }
            }

            if path.is_dir() {
                let items = Arc::clone(&items);
                let is_searching = Arc::clone(&is_searching);
                Manager::fallback_recursion(term, path, items, is_searching, delta_time)
                    .unwrap_or(());
            }
        });
        Ok(())
    }

    /// Creates a folder or a file in the current directory.
    /// Filenames ending with "/" are considered folders.
    ///
    /// *file_name* is a '&str' because it is easier just to specify a name
    /// and then let the function build the correct path based on the current directory.
    pub fn create_fsitem(&self, file_name: &str) -> Result<(), String> {
        let mut path = PathBuf::new();
        path.push(&self.current);
        path.push(file_name);

        if file_name.ends_with('/') {
            match fs::create_dir_all(&path) {
                Ok(_) => Ok(()),
                Err(e) => Err(e.to_string()),
            }
        } else {
            match fs::File::create_new(path) {
                Ok(_) => Ok(()),
                Err(e) => Err(e.to_string()),
            }
        }
    }

    /// Deletes the file or folder specified.
    /// Folders are deleted recursively., path.exists()
    ///
    /// This fn is the inverse of create_fsitem():
    /// Here it is easier to specify a path (that you can get from an fsitem.get_path()).
    /// This is less error prone, since the API keeps track of paths corresponding to an item.
    pub fn delete_fsitem(&self, path: &PathBuf) -> Result<(), String> {
        if path.is_dir() {
            match fs::remove_dir_all(path) {
                Ok(_) => Ok(()),
                Err(e) => Err(e.to_string()),
            }
        } else {
            match fs::remove_file(path) {
                Ok(_) => Ok(()),
                Err(e) => Err(e.to_string()),
            }
        }
    }

    /// Renames an item in the folder
    pub fn rename_fsitem(&self, source: PathBuf, dest: &str) -> Result<(), String> {
        let mut temp = source.clone();
        temp.pop();
        temp.push(dest);

        if temp.exists() {
            return Err(String::from("Item with same name already exists"));
        }

        match fs::rename(source, temp) {
            Ok(_) => Ok(()),
            Err(e) => Err(e.to_string()),
        }
    }

    pub fn move_fsitem(
        &self,
        source: PathBuf,
        dest: PathBuf,
        option: MoveOption,
    ) -> Result<(), String> {
        if !source.exists() {
            return Err(String::from("Source doesn't exist"));
        }

        let mut dest = dest.clone();
        dest.push(source.file_name().unwrap());

        match option {
            MoveOption::Move => match fs::rename(&source, &dest) {
                Ok(_) => Ok(()),
                Err(_) => self.move_crossfs(source, dest, &option),
            },
            MoveOption::Copy => self.move_crossfs(source, dest, &option),
        }
    }

    /// Moves a file or folder to a different filesystem or mount point.
    /// It's basically a helper function for move_fsitem()
    fn move_crossfs(
        &self,
        source: PathBuf,
        dest: PathBuf,
        option: &MoveOption,
    ) -> Result<(), String> {
        //println!("Different mount point!: {source:?} {dest:?}");
        // [FILE] Copies 'src' to 'dest' then deletes 'src'
        if source.is_file() {
            match fs::copy(&source, dest) {
                Ok(_) => (),
                Err(e) => return Err(e.to_string()),
            }

            match option {
                MoveOption::Move => match fs::remove_file(&source) {
                    Ok(_) => Ok(()),
                    Err(e) => Err(e.to_string()),
                },
                MoveOption::Copy => Ok(()),
            }
        }
        // [DIR]
        else if source.is_dir() {
            match fs::create_dir_all(&dest) {
                Ok(_) => {
                    if let Ok(entries) = fs::read_dir(&source) {
                        for entry in entries.flatten() {
                            let fname = entry.file_name();
                            let dest_entry = dest.join(fname);
                            self.move_crossfs(entry.path(), dest_entry, option)?
                        }

                        match option {
                            MoveOption::Move => {
                                if let Err(e) = fs::remove_dir_all(source) {
                                    return Err(e.to_string());
                                }
                            }
                            MoveOption::Copy => (),
                        }
                        Ok(())
                    } else {
                        return Err(format!("Can't open dir: {}", dest.to_string_lossy()));
                    }
                }
                Err(e) => Err(e.to_string()),
            }
        } else {
            Err(String::from("Permission error"))
        }
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
        *self.flags.is_searching.lock().unwrap() = false;
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

        *self.flags.is_searching.lock().unwrap() = false;

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

        index.retain(|_, value| {
            value.retain(|path| {
                let metadata = path.metadata();
                let mut valid = true;
                if let Ok(metadata) = metadata {
                    let mod_time = metadata.modified().unwrap_or(SystemTime::now());
                    valid = mod_time
                        .elapsed()
                        .unwrap_or(Duration::from_secs(0))
                        .as_secs()
                        < Duration::from_secs(60 * 60 * 24 * 30 * 3).as_secs();
                }
                path.exists() && valid
            });
            !value.is_empty()
        });

        let mut idx_lock = self.index.lock().unwrap();
        idx_lock.index = index;
        idx_lock.last_sync = Some(last_sync);
        drop(idx_lock);

        Ok(())
    }
}
