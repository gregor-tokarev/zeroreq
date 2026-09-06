use gpui::{App, Menu, MenuItem};

use crate::zeroreq::actions::{CheckForUpdates, Quit};

pub fn use_menus(_: &mut App) -> Vec<Menu> {
    vec![Menu {
        name: "Zeroreq".into(),
        disabled: false,
        items: vec![
            MenuItem::action("About Zeroreq", workspace::OpenGeneralSettings),
            MenuItem::action("Check for Updates…", CheckForUpdates),
            MenuItem::separator(),
            MenuItem::action("Settings…", workspace::OpenSettings),
            MenuItem::separator(),
            MenuItem::action("Quit Zeroreq", Quit),
        ],
    }]
}

pub fn init(cx: &mut App) {
    let menus = use_menus(cx);
    cx.set_menus(menus);

    // AppKit's menu key equivalents are a snapshot of the keymap.
    cx.observe_global::<keybindings_service::KeybindingsService>(|cx| {
        let menus = use_menus(cx);
        cx.set_menus(menus);
    })
    .detach();
}
