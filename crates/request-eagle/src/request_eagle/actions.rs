use gpui_kit::{App, Entity, actions};

use super::quit;

#[cfg(test)]
mod tests;

actions!(request_eagle, [CheckForUpdates, Quit]);

pub fn init(updater: Entity<updater::Updater>, cx: &mut App) {
    keybindings_service::set_binding("cmd-q", Quit, None, cx)
        .expect("default quit keybinding should be valid");

    cx.on_action(move |_: &CheckForUpdates, cx| {
        updater.update(cx, |updater, cx| updater.check(cx));
        cx.defer(|cx| cx.dispatch_action(&workspace::OpenGeneralSettings));
    })
    .on_action(|_: &Quit, cx| quit::quit(cx));
}
