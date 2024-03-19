/// Scan the entire file system to update win-locate database.
///
/// Author: @antoninoLorenzo: https://github.com/antoninoLorenzo
/// Version: 0.0

use std::fs;
use std::env;
use std::fmt;
use std::process;
use std::fs::DirEntry;
use std::collections::HashMap;
use std::ops::Index;
use std::sync::{Arc, Mutex};
use std::path::{Path, PathBuf};
use std::time::{Instant, SystemTime};
use chrono::prelude::*;
use rayon::prelude::*;


/// Used by ItemFS
enum ItemType {
    DIR,
    FILE
}

/// ### ItemFS
/// Represents a file system item (file or directory) as it
/// will be sent in output to *win-locate* program.
// TODO: add reference (parent to son or vice-versa)
struct ItemFS {
    abs_path: String,
    name: String,
    f_type: ItemType,
    size: u128,
    last_edit: String
}

impl ItemFS {
    /// Constructor for ItemFS.
    ///
    /// **Parameters:**
    ///
    /// [abs_path](ItemFS::abs_path)   : absolute path as String
    ///
    /// [name](ItemFS::name)           : name of file or directory as String
    ///
    /// [f_type](ItemType)             : specify if item is file or directory
    ///
    /// [size](ItemFS::size)           : specify size in bytes
    ///
    /// [last_edit](ItemFS::last_edit) : specify date of last edit as String
    fn new(abs_path: String, name: String,
           f_type: ItemType, size: u128, last_edit: String) -> ItemFS {
        ItemFS {abs_path, name, f_type, size, last_edit}
    }

    /// Creates a ItemFS instance from a [DirEntry](std::fs::DirEntry) object.
    ///
    /// **Parameters:**
    ///
    /// [entry](std::fs::DirEntry)
    fn from_dir_entry(entry: &DirEntry) -> Result<ItemFS, String> {
        let path = entry.path();

        let meta = entry.metadata()
            .map_err(|e| e.to_string())?;
        let sys_time: SystemTime = meta.modified()
            .map_err(|e| e.to_string())?;

        let item_type = match path.is_dir() {
            true => ItemType::DIR,
            false => ItemType::FILE
        };

        let item: ItemFS = ItemFS::new(
            format_path(path),
            entry.file_name().to_string_lossy().to_string(),
            item_type,
            meta.len() as u128,
            convert_sys_time(sys_time)
        );

        Ok(item)
    }

    fn get_size(&self) -> u128 {
        self.size
    }
}

impl fmt::Display for ItemType {
    fn fmt(&self, f_type: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ItemType::DIR => write!(f_type, "DIR"),
            ItemType::FILE => write!(f_type, "FILE"),
        }
    }
}

impl fmt::Display for ItemFS {
    fn fmt(&self, item: &mut fmt::Formatter) -> fmt::Result {
        write!(
            item,
            "{}: {}\n{}: {} - {} bytes",
            self.f_type , self.abs_path, self.name, self.last_edit, self.size
        )
    }
}

/// Converts PathBuf to String (Windows)
#[cfg(target_os = "windows")]
fn format_path(p: PathBuf) -> String {
    p.as_path().display().to_string()
        .replace("\\", "/")
}

/// Converts PathBuf to String
#[cfg(not(target_os = "windows"))]
fn format_path(p: PathBuf) -> String {
    p.as_path().display().to_string()
}

/// Converts SystemTime to date String using specified format
///
/// **Parameters:**
///
/// [t](SystemTime)
///
/// [fmt](Option<&str>)
fn _convert_sys_time(t: SystemTime, fmt: Option<&str>) -> String {
    DateTime::<Utc>::from(t)
        .format(fmt.unwrap_or("%d-%m-%Y"))
        .to_string()
}

/// Converts SystemTime to date String using "%d-%m-%Y" format
///
/// **Parameters:**
///
/// [t](SystemTime)
fn convert_sys_time(t: SystemTime) -> String {
    _convert_sys_time(t, None)
}

///
fn scan_file_system(path: &Path, dir_sizes: Arc<Mutex<HashMap<String, u128>>>) -> Result<Vec<ItemFS>, String> {
    if !path.exists()
        || !fs::metadata(path).map_err(|e| e.to_string())?.is_dir() {
        return Err("Invalid path".to_string());
    }

    // TODO: how can I fix os error 5? (access denied)
    let entries = match fs::read_dir(path) {
        Ok(entries) => entries,
        Err(e) => return Err(format!("Failed to read directory: {e}")),
    };

    let indexed_entries: Result<Vec<_>, _> = entries
        .collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())?
        .par_iter()
        .map(|entry| {
            if entry.path().is_dir() {
                let item = ItemFS::from_dir_entry(&entry).expect("");

                // Scan subdirectory
                // TODO: fix os error 5
                let sub_entries = scan_file_system(
                    &entry.path(),
                    Arc::clone(&dir_sizes)
                ).expect("");

                // Add item to output
                Ok::<Vec<ItemFS>, String>(
                    vec![item]
                        .into_iter()
                        .chain(sub_entries.into_iter())
                        .collect()
                )
            } else {
                let item = ItemFS::from_dir_entry(&entry).expect("");

                // Update parent dir size
                let parent_str = entry.path().parent().unwrap().to_string_lossy().to_string();
                let parent_path = format_path(PathBuf::from(parent_str));
                let item_size = item.get_size();
                let mut dir_sizes = dir_sizes.lock().unwrap();
                *dir_sizes.entry(parent_path).or_insert(0) += item_size;

                Ok(vec![item])
            }
        })
        .collect();

    let mut index = Vec::new();
    for entry in indexed_entries? {
        index.extend(entry);
    }

    Ok(index)
}

/// Validates the path to *index.db* given in input
pub fn get_index_path(args: Vec<String>) -> Result<String, &'static str> {
    if args.len() != 2 {
        return Err("Must provide index path.");
        process::exit(1);
    }

    let path_str = Path::new(args.index(1))
        .to_path_buf()
        .to_string_lossy()
        .to_string();

    let index_path = Path::new(path_str.as_str());

    if !index_path.exists() {
        return Err("Invalid index path, doesn't exists.");
        process::exit(1);
    }

    if index_path.extension().expect("") != "db" {
        return Err("Invalid index path.");
        process::exit(1);
    }

    Ok(path_str)
}

/// Updates database at index path
fn update_index(items: Vec<ItemFS>) {
    // Rather than dropping the entire database and rebuilding it,
    // for an efficient update the following approach is taken:
    // 1. Get a Vec<ItemFS> from database
    // 2. Verify the following conditions:
    //
    // AbsPath(database) exists && AbsPath(items) exists
    //      2.1 drop from items vector (item already indexed)
    //
    // AbsPath(database) not exists && AbsPath(items) exists
    //      2.2 keep item (new item in the index path)
    //
    // AbsPath(database) exists && AbsPath(items) not exists
    //      2.3 the item was deleted from file system, drop from database
    //
    // 3. Add last items to database
}

fn main() {
    // Get index path
    let args: Vec<String> = env::args().collect();
    let index_path = match get_index_path(args) {
        Ok(result) => result,
        Err(out) => process::exit(1)
    };
    println!("{}", index_path);

    // Create a safe hashmap that can be
    // shared during file system scanning
    let dir_sizes = Arc::new(
            Mutex::new(HashMap::new())
    );

    let start = Instant::now();
    match scan_file_system(Path::new("C:/Users/anton/Desktop/test"), Arc::clone(&dir_sizes)) {
        Ok(result) => {

            let dir_sizes = dir_sizes.lock().unwrap();
            /*
            for (path, size) in dir_sizes.iter() {
                println!("Path: {}, Size: {} bytes", path, size);
            }

            for e in result {
                println!("{e}")
            }
            */
            // TODO: persist items to sqlite (check existence)
        }
        Err(e) => { println!("{e}")}
    }

    println!("Done in {}", start.elapsed().as_secs());
}
