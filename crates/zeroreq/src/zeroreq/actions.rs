use gpui::{App, KeyBinding, actions};

use super::{about, quit, updater};

actions!(zeroreq, [About, CheckForUpdates, Quit]);

pub fn init(cx: &mut App) {
    cx.bind_keys([KeyBinding::new("cmd-q", Quit, None)]);

    cx.on_action(|_: &About, cx| about::open_about_window(cx))
        .on_action(|_: &CheckForUpdates, cx| updater::open_update_window(cx))
        .on_action(|_: &Quit, cx| quit::quit(cx));
}
