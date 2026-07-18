use gpui::{App, Menu, MenuItem};

use crate::zeroreq::actions::{About, CheckForUpdates, Quit};

pub fn use_menus(_: &mut App) -> Vec<Menu> {
    vec![Menu {
        name: "Zeroreq".into(),
        disabled: false,
        items: vec![
            MenuItem::action("About Zeroreq", About),
            MenuItem::action("Check for Updates…", CheckForUpdates),
            MenuItem::separator(),
            MenuItem::action("Quit Zeroreq", Quit),
        ],
    }]
}
