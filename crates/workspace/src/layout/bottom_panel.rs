use gpui::*;
use gpui_component::{ActiveTheme as _, Colorize as _};

#[derive(Default, IntoElement)]
pub struct BottomPanel;

impl RenderOnce for BottomPanel {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        div()
            .border_t_1()
            .border_color(cx.theme().border)
            .flex()
            .flex_none()
            .items_center()
            .h_8()
            .bg(cx.theme().background.darken(0.20))
            .child("bottom panel")
    }
}
