use super::get_index_path;

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
