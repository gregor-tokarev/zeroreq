use gpui::{App, actions};

actions!(
    workspace,
    [
        ToggleLeftSidebar,
        OpenSettings,
        OpenGeneralSettings,
        CloseSettings
    ]
);

pub(crate) fn init(cx: &mut App) {
    keybindings_service::register(
        ToggleLeftSidebar,
        "Toggle sidebar",
        "Show or hide the collections sidebar.",
        "Workspace",
        Some("cmd-b"),
        None,
        cx,
    )
    .expect("default sidebar keybinding should be valid");

    keybindings_service::register(
        OpenSettings,
        "Open settings",
        "Open application settings.",
        "Settings",
        Some("cmd-,"),
        None,
        cx,
    )
    .expect("default settings keybinding should be valid");

    keybindings_service::register(
        CloseSettings,
        "Close settings",
        "Return to your workspace.",
        "Settings",
        Some("escape"),
        Some("Settings"),
        cx,
    )
    .expect("default close settings keybinding should be valid");
}
