use std::collections::HashMap;

use crate::Environment;

pub struct EnvironmentRegistry {
    entries: Vec<Environment>,
    index: HashMap<String, usize>,
}

impl EnvironmentRegistry {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            index: HashMap::new(),
        }
    }

    pub fn add(&mut self, environment: Environment) {
        let environment_index = self.entries.len();

        for name in environment.entries.keys() {
            self.index.entry(name.clone()).or_insert(environment_index);
        }

        self.entries.push(environment);
    }

    pub fn resolve(&self, name: &str) -> Option<&str> {
        let environment_index = self.index.get(name)?;
        self.entries[*environment_index].resolve(name)
    }
}

impl Default for EnvironmentRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    fn environment(entries: &[(&str, &str)]) -> Environment {
        Environment {
            path: PathBuf::new(),
            entries: entries
                .iter()
                .map(|(key, value)| ((*key).into(), (*value).into()))
                .collect(),
        }
    }

    #[test]
    fn resolves_entries_through_the_index() {
        let mut registry = EnvironmentRegistry::new();
        registry.add(environment(&[("shared", "first"), ("first_only", "one")]));
        registry.add(environment(&[("shared", "second"), ("second_only", "two")]));

        assert_eq!(registry.resolve("shared"), Some("first"));
        assert_eq!(registry.resolve("first_only"), Some("one"));
        assert_eq!(registry.resolve("second_only"), Some("two"));
        assert_eq!(registry.resolve("missing"), None);
    }
}
