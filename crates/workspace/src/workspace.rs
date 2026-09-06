use std::sync::Arc;

use crate::actions::{OpenGeneralSettings, OpenSettings, ToggleLeftSidebar};
use crate::layout::{bottom_panel::BottomPanel, sidebar::Sidebar, top_panel::TopPanel};
use crate::settings::{Settings, SettingsEvent, SettingsPage};
use collection::CollectionRegistry;
use gpui::*;
use gpui_component::{
    resizable::{h_resizable, resizable_panel},
    *,
};

struct Layout {
    collections: Arc<CollectionRegistry>,

    // The app owns version and update details; retain the page across settings visits.
    general_settings: AnyView,

    sidebar_visible: bool,

    settings: Option<Entity<Settings>>,
    previous_focus: Option<FocusHandle>,

    _settings_subscription: Option<Subscription>,
}

impl Layout {
    fn new(collections: Arc<CollectionRegistry>, general_settings: AnyView) -> Self {
        Self {
            collections,
            general_settings,
            sidebar_visible: true,
            settings: None,
            previous_focus: None,
            _settings_subscription: None,
        }
    }

    fn open_settings(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(settings) = &self.settings {
            settings.update(cx, |settings, cx| settings.focus(window, cx));
            return;
        }

        self.previous_focus = window.focused(cx);
        let settings = cx.new(|cx| Settings::new(self.general_settings.clone(), window, cx));

        self._settings_subscription = Some(cx.subscribe_in(
            &settings,
            window,
            |this, _, _: &SettingsEvent, window, cx| {
                this.settings = None;
                this._settings_subscription = None;

                if let Some(focus) = this.previous_focus.take() {
                    window.focus(&focus, cx);
                } else {
                    window.blur();
                }

                cx.notify();
            },
        ));

        self.settings = Some(settings);

        cx.notify();
    }

    fn toggle_sidebar(&mut self, cx: &mut Context<Self>) {
        self.sidebar_visible = !self.sidebar_visible;

        cx.notify();
    }
}

fn on_open_settings(layout: &Entity<Layout>, window: AnyWindowHandle, cx: &mut App) {
    let layout = layout.downgrade();
    let general_layout = layout.clone();

    cx.on_action(move |_: &OpenGeneralSettings, cx| {
        let layout = general_layout.clone();

        cx.defer(move |cx| {
            let _ = window.update(cx, |_, window, cx| {
                let _ = layout.update(cx, |this, cx| {
                    this.open_settings(window, cx);

                    if let Some(settings) = &this.settings {
                        settings.update(cx, |settings, cx| {
                            settings.select_page(SettingsPage::General, window, cx)
                        });
                    }
                });

                window.activate_window();
            });
        });
    });

    cx.on_action(move |_: &OpenSettings, cx| {
        let layout = layout.clone();

        cx.defer(move |cx| {
            let _ = window.update(cx, |_, window, cx| {
                let _ = layout.update(cx, |this, cx| this.open_settings(window, cx));

                window.activate_window();
            });
        });
    });
}

fn on_toggle_sidebar(layout: &Entity<Layout>, cx: &mut App) {
    let layout = layout.clone();

    cx.on_action(move |_: &ToggleLeftSidebar, cx| {
        layout.update(cx, |this, cx| this.toggle_sidebar(cx));
    });
}

impl Render for Layout {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        if let Some(settings) = &self.settings {
            return settings.clone().into_any_element();
        }

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
            .into_any_element()
    }
}

pub fn init(collections: CollectionRegistry, general_settings: AnyView, cx: &mut App) {
    crate::actions::init(cx);

    let window_options = crate::window_options::use_window_options(cx);
    let layout = cx.new(|_| Layout::new(Arc::new(collections), general_settings));
    on_toggle_sidebar(&layout, cx);

    cx.open_window(window_options, move |window, cx| {
        crate::window_options::use_compact_window_controls(window);
        on_open_settings(&layout, window.window_handle(), cx);

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
    use gpui::{AppContext as _, Modifiers, TestAppContext};
    use std::sync::Arc;

    #[gpui::test]
    fn toggle_sidebar_action(cx: &mut TestAppContext) {
        cx.update(|cx| {
            gpui_component::init(cx);
            crate::actions::init(cx);
        });

        let (layout, cx) = cx.add_window_view(|_, cx| {
            Layout::new(
                Arc::new(CollectionRegistry::new()),
                cx.new(|_| gpui::Empty).into(),
            )
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
