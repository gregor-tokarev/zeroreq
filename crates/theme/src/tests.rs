use super::{apply, init};
use gpui_kit::component::Theme;
use gpui_kit::{TestAppContext, base};

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
