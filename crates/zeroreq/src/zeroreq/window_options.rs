use gpui::{App, TitlebarOptions, WindowKind, WindowOptions, px};

pub fn use_window_options(_: &mut App) -> WindowOptions {
    WindowOptions {
        titlebar: Some(TitlebarOptions {
            title: None,
            appears_transparent: true,
            traffic_light_position: None,
        }),
        window_bounds: None,
        focus: false,
        is_movable: true,
        kind: WindowKind::Normal,
        window_min_size: Some(gpui::Size {
            width: px(360.0),
            height: px(240.0),
        }),
        ..Default::default()
    }
}
