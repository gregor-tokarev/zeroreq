use super::AppearanceSettings;
use gpui_kit::TestAppContext;
use std::time::Instant;

#[gpui_kit::test]
#[ignore = "manual appearance render benchmark"]
fn appearance_render_benchmark(cx: &mut TestAppContext) {
    cx.update(|cx| {
        gpui_kit::init(cx);
        request_eagle_theme::init(cx);
    });

    let (page, cx) = cx.add_window_view(AppearanceSettings::new);
    for (width, height) in [(1024., 768.), (1440., 900.), (3440., 1410.)] {
        cx.simulate_resize(gpui_kit::size(gpui_kit::px(width), gpui_kit::px(height)));
        cx.run_until_parked();

        let mut draws = Vec::new();
        let mut scrolls = Vec::new();

        for index in 0..140 {
            let duration = cx.update(|window, cx| {
                window.refresh();
                let started = Instant::now();
                window.draw(cx).clear(cx);
                started.elapsed()
            });

            if index >= 20 {
                draws.push(duration);
            }
        }

        for index in 0..140 {
            let duration = cx.update(|window, cx| {
                page.read(cx)
                    .list
                    .scroll_by(gpui_kit::px(if index % 80 < 40 { 24. } else { -24. }));
                window.refresh();
                let started = Instant::now();
                window.draw(cx).clear(cx);
                started.elapsed()
            });

            if index >= 20 {
                scrolls.push(duration);
            }
        }

        for (label, mut samples) in [("draw", draws), ("scroll", scrolls)] {
            samples.sort();
            let mean = samples
                .iter()
                .map(|duration| duration.as_secs_f64())
                .sum::<f64>()
                * 1000.
                / samples.len() as f64;
            eprintln!(
                "{width}x{height} {label}: mean {mean:.2} ms, p95 {:?}, p99 {:?}, max {:?}",
                samples[113], samples[118], samples[119]
            );
        }
    }
}

#[gpui_kit::test]
fn preview_cards_apply_the_selected_palette(cx: &mut TestAppContext) {
    cx.update(|cx| {
        gpui_kit::init(cx);
        request_eagle_theme::init(cx);
    });

    let (_, cx) = cx.add_window_view(AppearanceSettings::new);

    for (selector, expected_light, expected_dark) in [
        ("theme-Catppuccin Latte", "Catppuccin Latte", "Ayu Dark"),
        ("appearance-Light", "Catppuccin Latte", "Ayu Dark"),
    ] {
        let bounds = cx
            .debug_bounds(selector)
            .expect("visible appearance control");
        cx.simulate_click(bounds.center(), gpui_kit::Modifiers::default());
        cx.run_until_parked();

        cx.read(|cx| {
            let preferences = preferences::get(cx).appearance;
            assert_eq!(preferences.light_theme, expected_light);
            assert_eq!(preferences.dark_theme, expected_dark);
        });
    }

    cx.read(|cx| {
        let theme = gpui_kit::component::Theme::global(cx);
        assert!(!theme.is_dark());
        assert_eq!(theme.light_theme.name, "Catppuccin Latte");
    });
}

#[gpui_kit::test]
fn catalog_virtualizes_rows_and_reflows_on_resize(cx: &mut TestAppContext) {
    cx.update(|cx| {
        gpui_kit::init(cx);
        request_eagle_theme::init(cx);
    });

    let (page, cx) = cx.add_window_view(AppearanceSettings::new);
    assert!(cx.debug_bounds("theme-Twilight").is_none());

    cx.update(|_, cx| {
        page.update(cx, |page, cx| {
            page.list.scroll_to(gpui_kit::ListOffset {
                item_ix: page.rows.len() - 1,
                offset_in_item: gpui_kit::px(0.),
            });
            cx.notify();
        });
    });
    cx.run_until_parked();
    assert!(cx.debug_bounds("theme-Twilight").is_some());
    assert!(cx.debug_bounds("appearance-Light").is_none());

    for (width, expected_columns) in [(360., 1), (1024., 4)] {
        cx.simulate_resize(gpui_kit::size(gpui_kit::px(width), gpui_kit::px(768.)));
        cx.run_until_parked();
        cx.read(|cx| assert_eq!(page.read(cx).columns, expected_columns));
        assert!(cx.debug_bounds("appearance-Light").is_some());
    }
}

#[gpui_kit::test]
fn keyboard_can_reach_offscreen_themes(cx: &mut TestAppContext) {
    cx.update(|cx| {
        gpui_kit::init(cx);
        request_eagle_theme::init(cx);
    });
    let (page, cx) = cx.add_window_view(AppearanceSettings::new);

    cx.update(|window, cx| {
        let page = page.read(cx);
        let index = page
            .previews
            .iter()
            .position(|preview| preview.name == "Default Light")
            .unwrap();
        let focus = page.theme_focus[index].clone();
        window.focus(&focus, cx);
    });

    for _ in 0..38 {
        cx.simulate_keystrokes("tab");
    }
    let keystroke = gpui_kit::Keystroke::parse("enter").unwrap();
    cx.simulate_event(gpui_kit::KeyDownEvent {
        keystroke: keystroke.clone(),
        is_held: false,
        prefer_character_input: false,
    });
    cx.simulate_event(gpui_kit::KeyUpEvent { keystroke });
    cx.read(|cx| assert_eq!(preferences::get(cx).appearance.dark_theme, "Twilight"));
    assert!(cx.debug_bounds("theme-Twilight").is_some());
}
