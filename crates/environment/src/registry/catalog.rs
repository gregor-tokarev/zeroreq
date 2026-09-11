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
