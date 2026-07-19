use std::{
    fs, io,
    path::{Path, PathBuf},
};

use environment::Environment;
use thiserror::Error;
use toml_edit::{DocumentMut, Item, Table};

use crate::{DirEntry, Entry, FileEntry};

pub struct Collection {
    pub path: PathBuf,
    pub entries: Vec<Entry>,
    local_env: Environment,
}

impl Collection {
    pub fn from_path(
        path: impl AsRef<Path>,
        local_env: Environment,
    ) -> Result<Self, CollectionLoadError> {
        let path = path.as_ref();
        let metadata = fs::metadata(path).map_err(|source| CollectionLoadError::Read {
            path: path.to_path_buf(),
            source,
        })?;
        if !metadata.is_dir() {
            return Err(CollectionLoadError::UnsupportedPath {
                path: path.to_path_buf(),
            });
        }

        let entries = load_directory(path, Some(&local_env.path))?.entries;

        Ok(Self {
            path: path.to_path_buf(),
            entries,
            local_env,
        })
    }

    pub fn save_files(&mut self) -> Result<(), CollectionSaveError> {
        fs::create_dir_all(&self.path).map_err(|source| CollectionSaveError::Write {
            path: self.path.clone(),
            source,
        })?;

        for entry in &mut self.entries {
            save_entry(entry)?;
        }

        Ok(())
    }

    pub fn local_env(&self) -> &Environment {
        &self.local_env
    }
}

fn load_entry(path: &Path, excluded_path: Option<&Path>) -> Result<Entry, CollectionLoadError> {
    let metadata = fs::metadata(path).map_err(|source| CollectionLoadError::Read {
        path: path.to_path_buf(),
        source,
    })?;

    if metadata.is_dir() {
        load_directory(path, excluded_path).map(Entry::Directory)
    } else if metadata.is_file() {
        load_file(path).map(Entry::File)
    } else {
        Err(CollectionLoadError::UnsupportedPath {
            path: path.to_path_buf(),
        })
    }
}

fn load_directory(
    path: &Path,
    excluded_path: Option<&Path>,
) -> Result<DirEntry, CollectionLoadError> {
    let directory = fs::read_dir(path).map_err(|source| CollectionLoadError::Read {
        path: path.to_path_buf(),
        source,
    })?;

    let mut paths = directory
        .map(|entry| {
            entry
                .map(|entry| entry.path())
                .map_err(|source| CollectionLoadError::Read {
                    path: path.to_path_buf(),
                    source,
                })
        })
        .collect::<Result<Vec<_>, _>>()?;
    paths.sort();

    let mut entries = Vec::new();
    for child_path in paths {
        if excluded_path == Some(child_path.as_path()) {
            continue;
        }

        let file_type = fs::symlink_metadata(&child_path)
            .map_err(|source| CollectionLoadError::Read {
                path: child_path.clone(),
                source,
            })?
            .file_type();

        if file_type.is_dir()
            || (file_type.is_file()
                && child_path
                    .extension()
                    .and_then(|extension| extension.to_str())
                    == Some("toml"))
        {
            entries.push(load_entry(&child_path, excluded_path)?);
        }
    }

    Ok(DirEntry {
        path: path.to_path_buf(),
        name: file_name(path),
        entries,
    })
}

fn load_file(path: &Path) -> Result<FileEntry, CollectionLoadError> {
    let raw_content = fs::read_to_string(path).map_err(|source| CollectionLoadError::Read {
        path: path.to_path_buf(),
        source,
    })?;
    let mut entry: FileEntry =
        toml::from_str(&raw_content).map_err(|source| CollectionLoadError::Parse {
            path: path.to_path_buf(),
            source,
        })?;

    entry.path = path.to_path_buf();
    entry.raw_content = raw_content;
    Ok(entry)
}

fn save_entry(entry: &mut Entry) -> Result<(), CollectionSaveError> {
    match entry {
        Entry::File(file) => save_file(file),
        Entry::Directory(directory) => {
            fs::create_dir_all(&directory.path).map_err(|source| CollectionSaveError::Write {
                path: directory.path.clone(),
                source,
            })?;

            for entry in &mut directory.entries {
                save_entry(entry)?;
            }

            Ok(())
        }
    }
}

fn save_file(entry: &mut FileEntry) -> Result<(), CollectionSaveError> {
    if let Some(parent) = entry.path.parent() {
        fs::create_dir_all(parent).map_err(|source| CollectionSaveError::Write {
            path: parent.to_path_buf(),
            source,
        })?;
    }

    let rendered =
        toml::to_string_pretty(entry).map_err(|source| CollectionSaveError::Serialize {
            path: entry.path.clone(),
            source,
        })?;
    let updates = rendered
        .parse::<DocumentMut>()
        .map_err(|source| CollectionSaveError::Edit {
            path: entry.path.clone(),
            source,
        })?;
    let mut document =
        entry
            .raw_content
            .parse::<DocumentMut>()
            .map_err(|source| CollectionSaveError::Edit {
                path: entry.path.clone(),
                source,
            })?;

    merge_table(document.as_table_mut(), updates.as_table());

    let raw_content = document.to_string();
    fs::write(&entry.path, &raw_content).map_err(|source| CollectionSaveError::Write {
        path: entry.path.clone(),
        source,
    })?;
    entry.raw_content = raw_content;

    Ok(())
}

fn merge_table(target: &mut Table, updates: &Table) {
    for (key, update) in updates.iter() {
        if let Some(current) = target.get_mut(key) {
            merge_item(current, update);
        } else {
            target.insert(key, update.clone());
        }
    }
}

fn merge_item(target: &mut Item, update: &Item) {
    match (target, update) {
        (Item::Table(target), Item::Table(update)) => merge_table(target, update),
        (target, update) => *target = update.clone(),
    }
}

fn file_name(path: &Path) -> String {
    path.file_name()
        .unwrap_or(path.as_os_str())
        .to_string_lossy()
        .into_owned()
}

#[derive(Debug, Error)]
pub enum CollectionLoadError {
    #[error("failed to read {}: {source}", .path.display())]
    Read { path: PathBuf, source: io::Error },
    #[error("failed to parse {}: {source}", .path.display())]
    Parse {
        path: PathBuf,
        source: toml::de::Error,
    },
    #[error("unsupported collection path {}", .path.display())]
    UnsupportedPath { path: PathBuf },
}

#[derive(Debug, Error)]
pub enum CollectionSaveError {
    #[error("failed to serialize collection file {}: {source}", .path.display())]
    Serialize {
        path: PathBuf,
        source: toml::ser::Error,
    },
    #[error("failed to edit collection file {}: {source}", .path.display())]
    Edit {
        path: PathBuf,
        source: toml_edit::TomlError,
    },
    #[error("failed to write {}: {source}", .path.display())]
    Write { path: PathBuf, source: io::Error },
}

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, time::SystemTime};

    use super::*;
    use crate::{Method, Request};

    fn test_directory() -> PathBuf {
        std::env::temp_dir().join(format!(
            "zeroreq-collection-{}-{}",
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
}
