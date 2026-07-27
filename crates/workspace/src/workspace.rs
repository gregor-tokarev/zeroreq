use std::sync::Arc;

use crate::layout::{bottom_panel::BottomPanel, sidebar::Sidebar, top_panel::TopPanel};
use collection::CollectionRegistry;
use gpui::*;
use gpui_component::{
    resizable::{h_resizable, resizable_panel},
    *,
};

struct Layout {
    collections: Arc<CollectionRegistry>,
}

impl Render for Layout {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .size_full()
            .child(TopPanel)
            .child(
                div().flex_1().min_h_0().child(
                    h_resizable("main_split")
                        .child(
                            resizable_panel()
                                .size_range(px(200.)..px(400.))
                                .child(Sidebar::new(Arc::clone(&self.collections))),
                        )
                        .child(div().child("right panel").into_any_element()),
                ),
            )
            .child(BottomPanel)
    }
}

pub fn init(collections: CollectionRegistry, cx: &mut App) {
    let window_options = crate::window_options::use_window_options(cx);
    let collections = Arc::new(collections);

    cx.open_window(window_options, move |window, cx| {
        crate::window_options::use_compact_window_controls(window);
        let view = cx.new(|_| Layout { collections });
        cx.new(|cx| Root::new(view, window, cx).bg(cx.theme().background))
    })
    .expect("Failed to open the window");
}
