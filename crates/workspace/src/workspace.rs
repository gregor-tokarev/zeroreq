use std::sync::Arc;

use crate::actions::ToggleLeftSidebar;
use crate::layout::{bottom_panel::BottomPanel, sidebar::Sidebar, top_panel::TopPanel};
use collection::CollectionRegistry;
use gpui::*;
use gpui_component::{
    resizable::{h_resizable, resizable_panel},
    *,
};

struct Layout {
    collections: Arc<CollectionRegistry>,
    sidebar_visible: bool,
}

impl Layout {
    fn toggle_sidebar(&mut self, cx: &mut Context<Self>) {
        self.sidebar_visible = !self.sidebar_visible;
        cx.notify();
    }
}

fn on_toggle_sidebar(layout: &Entity<Layout>, cx: &mut App) {
    let layout = layout.clone();
    cx.on_action(move |_: &ToggleLeftSidebar, cx| {
        layout.update(cx, |this, cx| this.toggle_sidebar(cx));
    });
}

impl Render for Layout {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .size_full()
            .child(TopPanel)
            .child(div().flex_1().min_h_0().child(if self.sidebar_visible {
                h_resizable("main_split")
                    .child(
                        resizable_panel()
                            .size_range(px(200.)..px(400.))
                            .child(Sidebar::new(Arc::clone(&self.collections))),
                    )
                    .child(div().child("right panel").into_any_element())
                    .into_any_element()
            } else {
                div().size_full().child("right panel").into_any_element()
            }))
            .child(BottomPanel::new(self.sidebar_visible))
    }
}

pub fn init(collections: CollectionRegistry, cx: &mut App) {
    crate::actions::init(cx);

    let window_options = crate::window_options::use_window_options(cx);
    let layout = cx.new(|_| Layout {
        collections: Arc::new(collections),
        sidebar_visible: true,
    });
    on_toggle_sidebar(&layout, cx);

    cx.open_window(window_options, move |window, cx| {
        crate::window_options::use_compact_window_controls(window);
        cx.new(|cx| Root::new(layout.clone(), window, cx).bg(cx.theme().background))
    })
    .expect("Failed to open the window");
}

#[cfg(test)]
mod tests {
    use super::{Layout, on_toggle_sidebar};
    use crate::actions::ToggleLeftSidebar;
    use crate::layout::bottom_panel::TOGGLE_SIDEBAR_BUTTON;
    use collection::CollectionRegistry;
    use gpui::{Modifiers, TestAppContext};
    use std::sync::Arc;

    #[gpui::test]
    fn toggle_sidebar_action(cx: &mut TestAppContext) {
        cx.update(|cx| {
            gpui_component::init(cx);
            crate::actions::init(cx);
        });

        let (layout, cx) = cx.add_window_view(|_, _| Layout {
            collections: Arc::new(CollectionRegistry::new()),
            sidebar_visible: true,
        });
        cx.update(|_, cx| on_toggle_sidebar(&layout, cx));

        let sidebar_visible = |cx: &TestAppContext| cx.read(|cx| layout.read(cx).sidebar_visible);

        assert!(sidebar_visible(cx));

        // The tooltip hint shows this binding.
        cx.update(|window, _| {
            let binding = window
                .highest_precedence_binding_for_action(&ToggleLeftSidebar)
                .expect("ToggleLeftSidebar should be bound");
            assert_eq!(binding.keystrokes()[0].inner().to_string(), "⌘B");
        });

        cx.simulate_keystrokes("cmd-b");
        assert!(!sidebar_visible(cx));

        cx.simulate_keystrokes("cmd-b");
        assert!(sidebar_visible(cx));

        let button_bounds = cx
            .debug_bounds(TOGGLE_SIDEBAR_BUTTON)
            .expect("toggle-sidebar button should be rendered");
        cx.simulate_click(button_bounds.center(), Modifiers::default());
        assert!(!sidebar_visible(cx));
    }
}
