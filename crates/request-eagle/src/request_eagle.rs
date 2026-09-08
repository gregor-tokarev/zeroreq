pub mod actions;
pub mod assets;
pub mod menu;
pub mod quit;

/// Move existing settings and collections before any service opens them.
pub fn migrate_data_directory(home: &std::path::Path) -> std::io::Result<()> {
    let old = home.join(".zeroreq");
    let new = home.join(".request-eagle");

    if old.try_exists()? && !new.try_exists()? {
        std::fs::rename(old, new)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{fs, time::SystemTime};

    #[test]
    fn migrates_saved_data_once_and_preserves_existing_destination() {
        let home = std::env::temp_dir().join(format!(
            "request-eagle-migration-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let old = home.join(".zeroreq");
        let new = home.join(".request-eagle");

        migrate_data_directory(&home).unwrap();
        assert!(!new.exists());

        fs::create_dir_all(old.join("collections")).unwrap();
        fs::write(old.join("keybindings.json"), "settings").unwrap();
        fs::write(old.join("collections/request.toml"), "request").unwrap();

        migrate_data_directory(&home).unwrap();
        migrate_data_directory(&home).unwrap();

        assert!(!old.exists());
        assert_eq!(
            fs::read_to_string(new.join("keybindings.json")).unwrap(),
            "settings"
        );
        assert_eq!(
            fs::read_to_string(new.join("collections/request.toml")).unwrap(),
            "request"
        );

        fs::create_dir_all(&old).unwrap();
        fs::write(old.join("keybindings.json"), "legacy").unwrap();
        migrate_data_directory(&home).unwrap();

        assert_eq!(
            fs::read_to_string(new.join("keybindings.json")).unwrap(),
            "settings"
        );
        assert_eq!(
            fs::read_to_string(old.join("keybindings.json")).unwrap(),
            "legacy"
        );

        fs::remove_dir_all(home).unwrap();
    }
}
