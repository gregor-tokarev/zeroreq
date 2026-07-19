//! Theme setup shared by Zeroreq windows.

use gpui::{App, SharedString};
use gpui_component::{Theme, ThemeRegistry};

/// The custom theme applied when the application starts.
pub const DEFAULT_THEME: &str = "Ayu Dark";

const EMBEDDED_THEME_SETS: &[&str] = &[
    include_str!("../themes/adventure.json"),
    include_str!("../themes/alduin.json"),
    include_str!("../themes/asciinema.json"),
    include_str!("../themes/aurora.json"),
    include_str!("../themes/ayu.json"),
    include_str!("../themes/catppuccin.json"),
    include_str!("../themes/everforest.json"),
    include_str!("../themes/fahrenheit.json"),
    include_str!("../themes/flexoki.json"),
    include_str!("../themes/gruvbox.json"),
    include_str!("../themes/harper.json"),
    include_str!("../themes/hybrid.json"),
    include_str!("../themes/jellybeans.json"),
    include_str!("../themes/kibble.json"),
    include_str!("../themes/macos-classic.json"),
    include_str!("../themes/matrix.json"),
    include_str!("../themes/mellifluous.json"),
    include_str!("../themes/molokai.json"),
    include_str!("../themes/solarized.json"),
    include_str!("../themes/spaceduck.json"),
    include_str!("../themes/tokyonight.json"),
    include_str!("../themes/twilight.json"),
];

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
