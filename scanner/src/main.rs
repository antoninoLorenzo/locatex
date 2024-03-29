mod tests;

/// Scan the entire file system to update win-locate database.
///
/// Author: @antoninoLorenzo: https://github.com/antoninoLorenzo
/// Version: 0.0

use std::fs;
use std::env;
use std::fmt;
use std::process;
use std::fs::DirEntry;
use std::str::FromStr;
use std::collections::HashMap;
use std::fmt::{Formatter};
use std::ops::Index;
use std::sync::{Arc, Mutex};
use std::path::{Path, PathBuf};
use std::time::{Instant, SystemTime};
use chrono::prelude::*;
use rayon::prelude::*;
use rusqlite::*;


/// Used by ItemFS
pub enum ItemType {
    DIR,
    FILE
}

/// ### ItemFS
/// Represents a file system item (file or directory) as it
/// will be sent in output to *win-locate* program.
// TODO: add reference (parent to son or vice-versa)
pub struct ItemFS {
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
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match *self {
            ItemType::DIR => write!(f, "directory"),
            ItemType::FILE => write!(f, "file")
        }
    }
}

impl fmt::Display for ItemFS {
    fn fmt(&self, item: &mut fmt::Formatter) -> fmt::Result {
        write!(
            item,
            "{}: {}\n{}: {} - {} bytes",
            self.f_type.to_string() , self.abs_path, self.name, self.last_edit, self.size
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

/// Starts from provided *path* and performs recursive scan
///
/// **Parameters:**
///
/// [path](Path)  : start path
///
/// [dir_sizes]() : dir sizes is basically a hashmap {abs_path: size} used
///                 because with Metadata::len the directory size is always 0 (...)
///
///                 std::sync::Arc is used to provide shared ownership
///                 std::sync::Mutex is used to perform updates with no race conditions
///
/// **Return**
///
/// Returns a vector of ItemFS or the error string
pub fn scan_file_system(path: &Path, dir_sizes: Arc<Mutex<HashMap<String, u128>>>) -> Result<Vec<ItemFS>, String> {
    if !path.exists()
        || !fs::metadata(path).map_err(|e| e.to_string())?.is_dir() {
        return Err("Invalid path".to_string());
    }

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
                if let Ok(res) = scan_file_system(
                    &entry.path(),
                    Arc::clone(&dir_sizes)) {

                    // Add item to output
                    return  Ok::<Vec<ItemFS>, String>(
                        vec![item]
                            .into_iter()
                            .chain(res.into_iter())
                            .collect()
                    )
                } else {
                    Ok(vec![]) // just return nothing
                }
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


/// Updates database at index path
pub fn update_index(items: Vec<ItemFS>, db_path: &String) -> Result<(), String> {
    // TODO
    //  implement logic to exclude existing items
    //  implement logic to remove deleted items
    
    let mut conn: Connection = match Connection::open(db_path) {
        Ok(connection) => { connection },
        Err(..) => {
            return Err("Failed acquiring connection.".to_string())
        }
    };

    // This is basically a try catch
    let result = (|| {
        let tx: Transaction = match conn.transaction() {
            Ok(transaction) => { transaction },
            Err(..) => {
                return Err("Failed acquiring transaction.".to_string())
            }
        };

        // ...

        for item in items {
            match tx.execute(
                "INSERT INTO fs (AbsPath, Name, Size, LastEdit, Type) \
            VALUES (?, ?, ?, ?, ?)",
                &[
                    &item.abs_path,
                    &item.name,
                    &item.size.to_string(),
                    &item.last_edit,
                    &item.f_type.to_string()
                ]
            ) {
                Ok(..) => {},
                Err(err) => {
                    return Err(err.to_string())
                }
            };
        }

        match tx.commit() {
            Ok(..) => Ok(()),
            Err(err) => {
                return Err(err.to_string())
            }
        }
    })();

    match result {
        Ok(..) => {},
        Err(err) => {
            conn.close().expect("PANIC: can't close connection");
            return Err(err.to_string())
        }
    }

    conn.close().expect("PANIC: can't close connection");
    Ok(())
}

/// Validates the path to *index.db* given in input
pub fn get_index_path(args: Vec<String>) -> Result<String, &'static str> {
    if args.len() != 2 {
        return Err("Must provide index path.");
    }

    let path_str = Path::new(args.index(1))
        .to_path_buf()
        .to_string_lossy()
        .to_string();

    let index_path = Path::new(path_str.as_str());

    if !index_path.exists() {
        return Err("Invalid index path, doesn't exists.");
    }

    if index_path.extension().expect("") != "db" {
        return Err("Invalid index path.");
    }

    Ok(path_str)
}

fn get_exclusions() {
    // TODO: parse a config file without adding dependencies
}

fn main() {
    // Get index path
    let args: Vec<String> = env::args().collect();
    let index_path = match get_index_path(args) {
        Ok(result) => result,
        Err(out) => {
            println!("Error: {out}");
            process::exit(1)
        }
    };
    println!("[i] Index Path: {}\n", index_path);

    // Create a safe hashmap that can be
    // shared during file system scanning
    let dir_sizes = Arc::new(
            Mutex::new(HashMap::new())
    );

    println!("------------------------ START SCANNING ------------------------");
    let start = Instant::now();
    // TODO: replace path with machine independent
    match scan_file_system(Path::new("C:/"), Arc::clone(&dir_sizes)) {
        Ok(result) => {
            println!("Scanned in {}s\n", start.elapsed().as_secs());

            let _dir_sizes = dir_sizes.lock().unwrap();
            // TODO: merge dir_sizes with Vec<ItemFS>
            /*
            for (path, size) in dir_sizes.iter() {
                println!("Path: {}, Size: {} bytes", path, size);
            }

            for e in result {
                println!("{e}")
            }
            */
            println!("---------------------------- PERSIST ---------------------------");
            let start_commit = Instant::now();

            match update_index(result, &index_path) {
                Ok(..) => {
                    println!("Commit in {}s\n", start_commit.elapsed().as_secs());
                },
                Err(e) => {
                    println!("Error: {e}");
                }
            };

        }
        Err(e) => { println!("{e}")}
    }

}
