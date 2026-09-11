use std::{collections::HashMap, time::SystemTime};

use super::Collection;
use crate::{Entry, Method, Request};
use environment::Environment;
use std::{
    fs,
    path::{Path, PathBuf},
};

fn test_directory() -> PathBuf {
    std::env::temp_dir().join(format!(
        "request-eagle-collection-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ))
}

fn environment(path: &Path) -> Environment {
    Environment {
        path: path.to_path_buf(),
        entries: HashMap::new(),
    }
}

#[test]
fn loads_edits_and_saves_a_directory_without_losing_user_content() {
    let root = test_directory();
    let nested = root.join("users");
    let request_path = nested.join("list.toml");

    fs::create_dir_all(&nested).unwrap();
    fs::write(
        &request_path,
        r#"# user's request
id = "list-users"
name = "List users"
schema_version = 1
custom = "keep me"

[request]
type = "http"
method = "GET"
path = "/users"
headers = [["Accept", "application/json"]]
request_custom = "keep me too"
"#,
    )
    .unwrap();
    fs::write(root.join("environment.toml"), "base_url = \"local\"\n").unwrap();

    let mut collection =
        Collection::from_path(&root, environment(&root.join("environment.toml"))).unwrap();
    let Entry::Directory(users) = &mut collection.entries[0] else {
        panic!("expected users directory");
    };
    let Entry::File(request) = &mut users.entries[0] else {
        panic!("expected request file");
    };
    let Request::Http(request) = &mut request.request;

    assert!(matches!(request.method, Method::Get));

    request.path = "/v2/users".into();

    collection.save_files().unwrap();

    let saved = fs::read_to_string(&request_path).unwrap();
    assert!(saved.contains("# user's request"));
    assert!(saved.contains("custom = \"keep me\""));
    assert!(saved.contains("request_custom = \"keep me too\""));
    assert!(saved.contains("path = \"/v2/users\""));

    let reloaded =
        Collection::from_path(&root, environment(&root.join("environment.toml"))).unwrap();
    let Entry::Directory(users) = &reloaded.entries[0] else {
        panic!("expected users directory");
    };
    let Entry::File(request) = &users.entries[0] else {
        panic!("expected request file");
    };
    let Request::Http(request) = &request.request;

    assert_eq!(request.path, "/v2/users");

    fs::remove_dir_all(root).unwrap();
}
