use super::{
    ItemType,
    ItemFS,
    get_index_path,
    scan_file_system,
    update_index
};


// ------------------------------ get_index_path tests


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

// ------------------------------ ItemType tests


// ------------------------------ ItemFS tests



// ------------------------------ scan_file_system tests

#[test]
fn not_existing_target_directory() {

}
#[test]
fn empty_target_directory() {

}
#[test]
fn valid_target_directory() {

}
#[test]
fn permission_denied() {

}
#[test]
fn performance_test() {

}
