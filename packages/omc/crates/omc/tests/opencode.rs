use omc::cli::opencode;

#[test]
fn test_config_path_uses_dot_config() {
    let path = opencode::config_path().unwrap();
    let home = dirs::home_dir().expect("could not determine home directory");
    let config_dir = home.join(".config").join("opencode");

    assert!(
        path.starts_with(&config_dir),
        "config path should be under .config/opencode/, got: {}",
        path.display()
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
    let config_dir = home.join(".config").join("opencode");

    assert!(
        path == config_dir.join("opencode.jsonc") || path == config_dir.join("opencode.json"),
        "config path should be opencode.jsonc or opencode.json under {config_dir:?}, got: {path:?}"
    );
}
