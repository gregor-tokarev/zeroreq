use gpui::{
    App, AppContext, Context, IntoElement, ParentElement, Render, Size, Styled, TitlebarOptions,
    Window, WindowBounds, WindowKind, WindowOptions, div, px,
};
use gpui_component::{ActiveTheme as _, StyledExt};

struct AboutWindow;

impl Render for AboutWindow {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .v_flex()
            .size_full()
            .bg(cx.theme().background)
            .text_color(cx.theme().foreground)
            .items_center()
            .justify_center()
            .gap_2()
            .child(div().text_size(px(26.)).child("Zeroreq"))
            .child(
                div()
                    .text_size(px(13.))
                    .text_color(cx.theme().muted_foreground)
                    .child(format!("Version {}", env!("CARGO_PKG_VERSION"))),
            )
    }
}

pub fn open_about_window(cx: &mut App) {
    if let Some(existing) = cx
        .windows()
        .into_iter()
        .find_map(|window| window.downcast::<AboutWindow>())
    {
        let _ = existing.update(cx, |_, window, _| window.activate_window());
        return;
    }

    let window_size = Size {
        width: px(420.),
        height: px(240.),
    };

    if let Err(error) = cx.open_window(
        WindowOptions {
            titlebar: Some(TitlebarOptions {
                title: Some("About Zeroreq".into()),
                appears_transparent: true,
                ..Default::default()
            }),
            window_bounds: Some(WindowBounds::centered(window_size, cx)),
            is_resizable: false,
            is_minimizable: false,
            kind: WindowKind::Floating,
            ..Default::default()
        },
        |_, cx| cx.new(|_| AboutWindow),
    ) {
        eprintln!("failed to open About Zeroreq window: {error}");
    }
}
