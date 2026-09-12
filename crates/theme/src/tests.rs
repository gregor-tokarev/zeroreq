use super::{apply, init};
use gpui_kit::component::Theme;
use gpui_kit::{TestAppContext, base};

#[gpui_kit::test]
fn catalog_contains_only_complete_switchable_pairs(cx: &mut TestAppContext) {
    cx.update(|cx| {
        gpui_kit::init(cx);
        init(cx);

        let registry = gpui_kit::component::ThemeRegistry::global(cx);
        assert_eq!(registry.themes().len(), crate::THEME_PAIRS.len() * 2);

        for &(light, dark) in crate::THEME_PAIRS {
            assert!(!registry.themes()[light].mode.is_dark(), "{light}");
            assert!(registry.themes()[dark].mode.is_dark(), "{dark}");
            assert_eq!(crate::theme_pair(light), Some((light, dark)));
            assert_eq!(crate::theme_pair(dark), Some((light, dark)));
        }

        for &(light, dark) in crate::THEME_PAIRS {
            for (name, is_dark) in [(light, false), (dark, true)] {
                assert!(apply(name, cx));
                assert_eq!(Theme::global(cx).is_dark(), is_dark);
                assert_eq!(Theme::global(cx).highlight_theme.name, name);
            }
        }
    });
}

#[gpui_kit::test]
fn startup_links_saved_themes_and_replaces_removed_variants(cx: &mut TestAppContext) {
    use preferences::AppearanceMode::{Dark, Light};

    for (mode, light, dark, expected_light, expected_dark) in [
        (
            Dark,
            "Ayu Light",
            "Catppuccin Frappe",
            "Catppuccin Latte",
            "Catppuccin Mocha",
        ),
        (
            Dark,
            "Ayu Light",
            "Catppuccin Macchiato",
            "Catppuccin Latte",
            "Catppuccin Mocha",
        ),
        (
            Dark,
            "Gruvbox Light",
            "Twilight",
            "Gruvbox Light",
            "Gruvbox Dark",
        ),
        (
            Light,
            "Aurora Light",
            "Solarized Dark",
            "Solarized Light",
            "Solarized Dark",
        ),
        (
            Light,
            "Aurora Light",
            "Catppuccin Frappe",
            "Catppuccin Latte",
            "Catppuccin Mocha",
        ),
        (Dark, "Aurora Light", "Twilight", "Ayu Light", "Ayu Dark"),
        (
            Light,
            "Catppuccin Latte",
            "Ayu Dark",
            "Catppuccin Latte",
            "Catppuccin Mocha",
        ),
        (
            Dark,
            "Catppuccin Latte",
            "Ayu Dark",
            "Ayu Light",
            "Ayu Dark",
        ),
    ] {
        cx.update(|cx| {
            gpui_kit::init(cx);
            preferences::update(cx, |preferences| {
                preferences.appearance.mode = mode;
                preferences.appearance.light_theme = light.into();
                preferences.appearance.dark_theme = dark.into();
            })
            .unwrap();

            init(cx);

            let preferences = &cx.global::<preferences::Preferences>().appearance;
            assert_eq!(preferences.light_theme, expected_light);
            assert_eq!(preferences.dark_theme, expected_dark);
            assert_eq!(
                Theme::global(cx).highlight_theme.name,
                if mode == Dark {
                    expected_dark
                } else {
                    expected_light
                }
            );
        });
    }
}

#[gpui_kit::test]
fn resize_handles_follow_the_applied_theme(cx: &mut TestAppContext) {
    cx.update(|cx| {
        gpui_kit::init(cx);
        init(cx);

        for name in ["Ayu Light", "Ayu Dark"] {
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
