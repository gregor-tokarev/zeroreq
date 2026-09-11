use std::path::PathBuf;

use super::EnvironmentRegistry;
use crate::Environment;

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
