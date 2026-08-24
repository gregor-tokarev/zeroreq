use gpui::{App, actions};

actions!(workspace, [ToggleLeftSidebar]);

pub(crate) fn init(cx: &mut App) {
    keybindings_service::set_binding("cmd-b", ToggleLeftSidebar, None, cx)
        .expect("default sidebar keybinding should be valid");
}
