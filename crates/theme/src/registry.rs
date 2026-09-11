use crate::apply_preferences;

use gpui_kit::component::{Theme, ThemeRegistry};
use gpui_kit::{App, SharedString};

include!(concat!(env!("OUT_DIR"), "/embedded_theme_sets.rs"));

pub fn init(cx: &mut App) {
    let registry = ThemeRegistry::global_mut(cx);

    for theme_set in EMBEDDED_THEME_SETS {
        registry
            .load_themes_from_str(theme_set)
            .expect("bundled gpui-component theme should be valid");
    }

    preferences::init(cx);
    cx.observe_global::<preferences::Preferences>(|cx| {
        apply_preferences(cx.window_appearance(), cx);
    })
    .detach();
    apply_preferences(cx.window_appearance(), cx);
}

pub fn apply(name: &str, cx: &mut App) -> bool {
    let name = SharedString::from(name);
    let Some(config) = ThemeRegistry::global(cx).themes().get(&name).cloned() else {
        return false;
    };

    Theme::global_mut(cx).apply_config(&config);
    Theme::sync_base(cx);

    cx.refresh_windows();

    true
}
