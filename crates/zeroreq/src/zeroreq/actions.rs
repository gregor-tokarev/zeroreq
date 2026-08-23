use gpui::{App, actions};

use super::{about, quit, updater};

actions!(zeroreq, [About, CheckForUpdates, Quit]);

pub fn init(cx: &mut App) {
    keybindings_service::set_binding("cmd-q", Quit, None, cx)
        .expect("default quit keybinding should be valid");

    cx.on_action(|_: &About, cx| about::open_about_window(cx))
        .on_action(|_: &CheckForUpdates, cx| updater::open_update_window(cx))
        .on_action(|_: &Quit, cx| quit::quit(cx));
}
