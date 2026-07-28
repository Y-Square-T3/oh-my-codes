use omc::cli::opencode;

#[test]
fn test_config_path_uses_dot_config() {
    let path = opencode::config_path().unwrap();
    let path_str = path.to_string_lossy();

    assert!(
        path_str.ends_with(".config/opencode/opencode.json"),
        "config path should end with .config/opencode/opencode.json, got: {path_str}"
    );
}

#[test]
fn test_config_path_is_under_home() {
    let path = opencode::config_path().unwrap();
    let home = dirs::home_dir().expect("could not determine home directory");

    assert!(
        path.starts_with(&home),
        "config path should be under home directory ({home:?}), got: {path:?}"
    );
}

#[test]
fn test_config_path_structure() {
    let path = opencode::config_path().unwrap();
    let home = dirs::home_dir().expect("could not determine home directory");
    let expected = home.join(".config").join("opencode").join("opencode.json");

    assert_eq!(path, expected);
}
