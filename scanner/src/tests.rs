use std::fs;
use std::path;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use super::{
    ItemType,
    ItemFS,
    get_index_path,
    scan_file_system,
    update_index
};


// get_index_path tests


#[test]
fn get_index_path_empty() {
    let args = vec![];

    let result = get_index_path(args);
    assert!(result.is_err());
}

#[test]
fn get_index_path_not_exists() {
    let mut args: Vec<String> = Vec::new();
    args.push("./hello".to_string());

    let result = get_index_path(args);
    assert!(result.is_err());
}

#[test]
fn get_index_path_invalid_db() {

}

#[test]
fn get_index_path_correct() {

}

// ItemType tests


// ItemFS tests



// scan_file_system tests

#[test]
fn not_existing_target_directory() {
    let path = path::Path::new("./hello_world");
    let dir_sizes = Arc::new(Mutex::new(HashMap::new()));

    let result = scan_file_system(path, Arc::clone(&dir_sizes));
    assert!(result.is_err())
}
#[test]
fn empty_target_directory() {
    fs::create_dir("./test_dir")
        .expect("[empty_target_directory] Error creating directory.");
    let path = path::Path::new("./test_dir");
    let dir_sizes =  Arc::new(Mutex::new(HashMap::new()));

    match scan_file_system(path, Arc::clone(&dir_sizes)) {
        Ok(res) => {
            fs::remove_dir("./test_dir")
                .expect("[empty_target_directory] Failed cleaning.");

            assert_eq!(res.len(), 0)
        },
        Err(err) => {
            fs::remove_dir("./test_dir")
                .expect("[empty_target_directory] Failed cleaning.");

            println!("[empty_target_directory]: {err}");
            assert_eq!(0, 1)
        }
    }
}
#[test]
fn valid_target_directory() {
    let path = path::Path::new("./src");
    let dir_sizes =  Arc::new(Mutex::new(HashMap::new()));

    match scan_file_system(path, Arc::clone(&dir_sizes)) {
        Ok(res) => {
            for item in &res {
                println!("{item}");
            }

            assert!(res.len() > 0)
        },
        Err(err) => {
            print!("[valid_target_directory]: {err}");
            assert_eq!(0, 1)
        }
    }
}
#[test]
fn permission_denied() {
    // TODO: use testable path from other systems
    let path = path::Path::new("C:\\Users\\anton");
    let dir_sizes =  Arc::new(Mutex::new(HashMap::new()));

    match scan_file_system(path, Arc::clone(&dir_sizes)) {
        Ok(res) => {
            assert!(res.len() > 0)
        },
        Err(err) => {
            print!("[permission_denied]: {err}");
            assert_eq!(0, 1)
        }
    }
}
#[test]
fn performance_test() {
    let path = path::Path::new("C:\\");
    let dir_sizes =  Arc::new(Mutex::new(HashMap::new()));

    let start = Instant::now();
    match scan_file_system(path, Arc::clone(&dir_sizes)) {
        Ok(res) => {
            println!("Scanned in {}s\n", start.elapsed().as_secs());

            assert!(res.len() > 0)
        },
        Err(err) => {
            print!("[permission_denied]: {err}");
            assert_eq!(0, 1)
        }
    }
}
