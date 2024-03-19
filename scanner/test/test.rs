use scanner::get_index_path;

#[test]
fn get_index_path_empty() {
    let args = vec![];
    let result = get_index_path(args);
    assert!(result.is_err());
}