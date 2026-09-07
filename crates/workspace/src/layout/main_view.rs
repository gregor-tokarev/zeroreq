use gpui_kit::*;

pub struct MainView;

impl Render for MainView {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div()
            .debug_selector(|| "main-view".into())
            .size_full()
            .child("main view")
    }
}
