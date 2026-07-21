use gpui::*;

#[derive(Default, IntoElement)]
pub struct Sidebar;

impl RenderOnce for Sidebar {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        div().size_full().child("sidebar")
    }
}
