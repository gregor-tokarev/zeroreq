use super::Environment;
use std::{
    fs,
    path::{Path, PathBuf},
};

#[test]
fn parses_entries() {
    let environment = Environment::from_toml(
        Path::new("development.toml"),
        r#"
            api_url = "https://api.example.com"
            api_token = "super-secret"
        "#,
    )
    .unwrap();

    assert_eq!(environment.path, PathBuf::from("development.toml"));
    assert_eq!(
        environment.resolve("api_url"),
        Some("https://api.example.com")
    );
    assert_eq!(environment.resolve("api_token"), Some("super-secret"));
    assert_eq!(environment.resolve("missing"), None);
}

#[test]
fn saves_and_loads_entries() {
    let path = std::env::temp_dir().join(format!(
        "request-eagle-environment-{}-save.toml",
        std::process::id()
    ));
    let environment = Environment::from_toml(
        &path,
        r#"
            second = "two"
            first = "one"
        "#,
    )
    .unwrap();

    environment.save_file().unwrap();
    let loaded = Environment::from_file(&path).unwrap();

    assert_eq!(loaded.resolve("first"), Some("one"));
    assert_eq!(loaded.resolve("second"), Some("two"));

    fs::remove_file(path).unwrap();
}
