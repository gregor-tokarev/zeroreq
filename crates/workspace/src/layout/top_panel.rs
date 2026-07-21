use gpui::*;
use gpui_component::ActiveTheme as _;

#[derive(Default, IntoElement)]
pub struct TopPanel;

impl RenderOnce for TopPanel {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        div()
            .border_b_1()
            .border_color(cx.theme().border)
            .flex()
            .flex_none()
            .items_center()
            .h(px(34.))
            .pl_20()
            .child("top panel")
    }
}
