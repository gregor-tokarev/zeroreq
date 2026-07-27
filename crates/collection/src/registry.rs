use std::{
    collections::HashMap,
    fs, io,
    path::{Path, PathBuf},
};

use environment::{Environment, EnvironmentLoadError};
use thiserror::Error;

use crate::{Collection, CollectionLoadError};

const ENVIRONMENT_FILE_NAME: &str = "environment.toml";

#[derive(Default)]
pub struct CollectionRegistry {
    collections: Vec<Collection>,
}

impl CollectionRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, collection: Collection) {
        self.collections.push(collection);
    }

    /// Loads collections from `~/.zeroreq/collections`.
    pub fn load() -> Result<Self, CollectionRegistryLoadError> {
        let home = dirs::home_dir().ok_or(CollectionRegistryLoadError::HomeDirectoryUnavailable)?;
        Self::from_path(home.join(".zeroreq").join("collections"))
    }

    pub fn from_path(path: impl AsRef<Path>) -> Result<Self, CollectionRegistryLoadError> {
        let path = path.as_ref();
        let directory = match fs::read_dir(path) {
            Ok(directory) => directory,
            Err(source) if source.kind() == io::ErrorKind::NotFound => return Ok(Self::new()),
            Err(source) => {
                return Err(CollectionRegistryLoadError::Read {
                    path: path.to_path_buf(),
                    source,
                });
            }
        };

        let mut collection_paths = directory
            .map(|entry| {
                entry.map(|entry| entry.path()).map_err(|source| {
                    CollectionRegistryLoadError::Read {
                        path: path.to_path_buf(),
                        source,
                    }
                })
            })
            .collect::<Result<Vec<_>, _>>()?;
        collection_paths.sort();

        let mut registry = Self::new();
        for collection_path in collection_paths {
            let metadata = fs::metadata(&collection_path).map_err(|source| {
                CollectionRegistryLoadError::Read {
                    path: collection_path.clone(),
                    source,
                }
            })?;
            if !metadata.is_dir() {
                continue;
            }

            let environment_path = collection_path.join(ENVIRONMENT_FILE_NAME);
            let environment = match Environment::from_file(&environment_path) {
                Ok(environment) => environment,
                Err(EnvironmentLoadError::Read { source, .. })
                    if source.kind() == io::ErrorKind::NotFound =>
                {
                    Environment {
                        path: environment_path,
                        entries: HashMap::new(),
                    }
                }
                Err(source) => {
                    return Err(CollectionRegistryLoadError::Environment {
                        path: environment_path,
                        source,
                    });
                }
            };

            let collection =
                Collection::from_path(&collection_path, environment).map_err(|source| {
                    CollectionRegistryLoadError::Collection {
                        path: collection_path,
                        source,
                    }
                })?;
            registry.add(collection);
        }

        Ok(registry)
    }

    pub fn collections(&self) -> &[Collection] {
        &self.collections
    }

    pub fn is_empty(&self) -> bool {
        self.collections.is_empty()
    }

    pub fn len(&self) -> usize {
        self.collections.len()
    }
}

#[derive(Debug, Error)]
pub enum CollectionRegistryLoadError {
    #[error("could not determine the user's home directory")]
    HomeDirectoryUnavailable,
    #[error("failed to read collections from {}: {source}", .path.display())]
    Read { path: PathBuf, source: io::Error },
    #[error("failed to load the environment for {}: {source}", .path.display())]
    Environment {
        path: PathBuf,
        source: EnvironmentLoadError,
    },
    #[error("failed to load collection {}: {source}", .path.display())]
    Collection {
        path: PathBuf,
        source: CollectionLoadError,
    },
}

impl FromIterator<Collection> for CollectionRegistry {
    fn from_iter<T: IntoIterator<Item = Collection>>(collections: T) -> Self {
        Self {
            collections: collections.into_iter().collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::SystemTime;

    use super::*;

    fn test_directory() -> PathBuf {
        std::env::temp_dir().join(format!(
            "zeroreq-collection-registry-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ))
    }

    #[test]
    fn loads_collection_directories_in_name_order() {
        let root = test_directory();
        let alpha = root.join("alpha");
        let beta = root.join("beta");
        fs::create_dir_all(&alpha).unwrap();
        fs::create_dir_all(&beta).unwrap();
        fs::write(alpha.join(ENVIRONMENT_FILE_NAME), "base_url = \"local\"\n").unwrap();
        fs::write(root.join("not-a-collection.toml"), "ignored = true\n").unwrap();

        let registry = CollectionRegistry::from_path(&root).unwrap();

        assert_eq!(registry.len(), 2);
        assert_eq!(registry.collections()[0].path, alpha);
        assert_eq!(registry.collections()[1].path, beta);
        assert_eq!(
            registry.collections()[0].local_env().resolve("base_url"),
            Some("local")
        );
        assert!(registry.collections()[1].local_env().entries.is_empty());

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn missing_collections_directory_loads_an_empty_registry() {
        let root = test_directory();

        let registry = CollectionRegistry::from_path(root).unwrap();

        assert!(registry.is_empty());
    }
}
