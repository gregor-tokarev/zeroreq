use gpui::*;
use gpui_component::ActiveTheme as _;

#[derive(Default)]
pub struct TopPanel;

impl Render for TopPanel {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
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
