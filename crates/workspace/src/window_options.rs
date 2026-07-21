use gpui::{App, TitlebarOptions, Window, WindowBounds, WindowKind, WindowOptions, point, px};

pub(crate) fn use_window_options(cx: &mut App) -> WindowOptions {
    let display = cx.primary_display();

    let display_id = display.as_ref().map(|display| display.id());
    let window_bounds = display.map(|display| WindowBounds::Maximized(display.default_bounds()));

    WindowOptions {
        titlebar: Some(TitlebarOptions {
            title: None,
            appears_transparent: true,
            traffic_light_position: Some(point(px(9.0), px(9.0))),
        }),
        window_bounds,
        focus: false,
        is_movable: true,
        kind: WindowKind::Normal,
        display_id,
        window_min_size: Some(gpui::Size {
            width: px(360.0),
            height: px(240.0),
        }),
        ..Default::default()
    }
}

pub(crate) fn use_compact_window_controls(window: &Window) {
    #[cfg(target_os = "macos")]
    use_compact_macos_window_controls(window);
}

#[cfg(target_os = "macos")]
fn use_compact_macos_window_controls(window: &Window) {
    use objc2::{runtime::NSObjectProtocol, sel};
    use objc2_app_kit::{NSView, NSWindowButton};
    use raw_window_handle::{HasWindowHandle, RawWindowHandle};

    let Ok(handle) = HasWindowHandle::window_handle(window) else {
        return;
    };
    let RawWindowHandle::AppKit(handle) = handle.as_raw() else {
        return;
    };

    // SAFETY: GPUI's AppKit raw-window handle contains its live native NSView.
    // The borrowed handle keeps the GPUI window alive for this function call.
    let native_view = unsafe { handle.ns_view.cast::<NSView>().as_ref() };
    let Some(native_window) = native_view.window() else {
        return;
    };

    for button_type in [
        NSWindowButton::CloseButton,
        NSWindowButton::MiniaturizeButton,
        NSWindowButton::ZoomButton,
    ] {
        let Some(button) = native_window.standardWindowButton(button_type) else {
            continue;
        };

        if button.respondsToSelector(sel!(setPrefersCompactControlSizeMetrics:)) {
            button.setPrefersCompactControlSizeMetrics(true);
        }
    }

    // Reapply Zed's placement after AppKit has switched to compact metrics.
    window.set_traffic_light_position(point(px(9.0), px(9.0)));
}
