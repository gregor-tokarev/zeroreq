//! Theme setup shared by Zeroreq windows.

use gpui::{App, SharedString};
use gpui_component::{Theme, ThemeRegistry};

/// The custom theme applied when the application starts.
pub const DEFAULT_THEME: &str = "Ayu Dark";

include!(concat!(env!("OUT_DIR"), "/embedded_theme_sets.rs"));

pub fn init(cx: &mut App) {
    let registry = ThemeRegistry::global_mut(cx);

    for theme_set in EMBEDDED_THEME_SETS {
        registry
            .load_themes_from_str(theme_set)
            .expect("bundled gpui-component theme should be valid");
    }

    assert!(
        apply(DEFAULT_THEME, cx),
        "default theme {DEFAULT_THEME:?} should be bundled"
    );
}

pub fn apply(name: &str, cx: &mut App) -> bool {
    let name = SharedString::from(name);
    let Some(config) = ThemeRegistry::global(cx).themes().get(&name).cloned() else {
        return false;
    };

    Theme::global_mut(cx).apply_config(&config);
    cx.refresh_windows();
    true
}

pub fn available_themes(cx: &App) -> Vec<SharedString> {
    ThemeRegistry::global(cx)
        .sorted_themes()
        .into_iter()
        .map(|theme| theme.name.clone())
        .collect()
}
