use gpui::{App, KeyBinding, actions};

pub mod about;
pub mod menu;
pub mod quit;
pub mod window_options;

actions!(zeroreq, [About, Quit]);

pub fn init(cx: &mut App) {
    cx.bind_keys([KeyBinding::new("cmd-q", Quit, None)]);

    cx.on_action(|_: &About, cx| about::open_about_window(cx))
        .on_action(|_: &Quit, cx| quit::quit(cx));
}
