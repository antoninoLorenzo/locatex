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
fn get_index_path_not_db() {
    fs::create_dir("./test_dir")
        .expect("[get_index_path_not_db] Failed creating directory");
    let mut args_dir: Vec<String> = Vec::new();
    args_dir.push("./test_dir".to_string());

    let result_dir = get_index_path(args_dir);
    assert!(result_dir.is_err());

    // Test without extension to see if it fails correctly
    let _file = fs::File::create("./test_dir/not_valid");
    let mut args_file: Vec<String> = Vec::new();
    args_file.push("./test_dir/not_valid".to_string());

    let result_file = get_index_path(args_file);

    fs::remove_file("./test_dir/not_valid")
        .expect("[get_index_path_not_db] Failed cleaning file.");
    fs::remove_dir("./test_dir")
        .expect("[get_index_path_not_db] Failed cleaning directory.");

    assert!(result_file.is_err());
}

/*
#[test]
fn get_index_path_correct() {

}
*/

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
fn pointing_to_file() {
    let _f = fs::File::create("./test_file");
    let path = path::Path::new("./test_file");
    let dir_sizes = Arc::new(Mutex::new(HashMap::new()));

    let result = scan_file_system(path, Arc::clone(&dir_sizes));

    fs::remove_file("./test_file")
        .expect("[pointing_to_file] Failed cleaning.");

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
