//! Theme setup shared by Request Eagle windows.

use gpui_kit::component::{Theme, ThemeRegistry};
use gpui_kit::{App, SharedString};

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
    Theme::sync_base(cx);

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

#[cfg(test)]
mod tests {
    use super::*;
    use gpui_kit::{TestAppContext, base};

    #[gpui_kit::test]
    fn resize_handles_follow_the_applied_theme(cx: &mut TestAppContext) {
        cx.update(|cx| {
            gpui_kit::init(cx);
            init(cx);

            for name in [DEFAULT_THEME, "Ayu Light", "Ayu Dark"] {
                assert!(apply(name, cx));

                let theme = Theme::global(cx);
                let base_theme = base::Theme::global(cx);

                assert_eq!(base_theme.resizable.handle, Some(theme.border), "{name}");
                assert_eq!(
                    base_theme.resizable.active_handle,
                    Some(theme.drag_border),
                    "{name}"
                );
            }
        });
    }
}
