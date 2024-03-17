/// Scan the entire file system to update win-locate database.
///
/// Author: @antoninoLorenzo: https://github.com/antoninoLorenzo
/// Version: 0.0

use std::fs;
use std::fmt;
use std::fs::DirEntry;
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

fn index_rayon(path: &Path) -> Result<Vec<String>, String> {
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
                let item = ItemFS::from_dir_entry(&entry);
                //println!("{}", item.expect("Error getting item."));

                match index_rayon(&entry.path()) {
                    Ok(sub_entries) => Ok::<Vec<String>, String>(sub_entries),
                    Err(_e) => {
                        // TODO: how can I fix os error 5? (access denied)
                        Ok(Vec::new())
                    },
                }
            } else {
                let item = ItemFS::from_dir_entry(&entry);
                //println!("{}", item.expect("Error getting item."));

                Ok(vec![entry.path().to_string_lossy().into_owned()])
            }
        })
        .collect();

    let mut index = Vec::new();
    for entry in indexed_entries? {
        index.extend(entry);
    }

    Ok(index)
}

fn main() {
    let start = Instant::now();

    match index_rayon(Path::new("C:/Users/anton")) {
        Ok(result) => {
            for _e in result {
                // println!("{e}")
            }
        }
        Err(e) => { println!("{e}")}
    }

    println!("Done in {}", start.elapsed().as_secs());
}
