use std::{sync::Arc, time::Duration};

use crate::actions::{OpenGeneralSettings, OpenSettings, ToggleLeftSidebar};
use crate::layout::{
    bottom_panel::BottomPanel, main_view::MainView, sidebar::Sidebar, top_panel::TopPanel,
};
use crate::settings::{Settings, SettingsEvent, SettingsPage};
use collection::CollectionRegistry;
use gpui_kit::base::motion::{self, Transition};
use gpui_kit::component::{
    animation::ease_in_out_cubic,
    resizable::{ResizableState, h_resizable, resizable_panel},
    *,
};
use gpui_kit::{prelude::FluentBuilder as _, *};
use updater::Updater;

struct Layout {
    top_panel: Entity<TopPanel>,
    sidebar: Entity<Sidebar>,
    main_view: Entity<MainView>,
    bottom_panel: Entity<BottomPanel>,

    main_split: Entity<ResizableState>,
    sidebar_visible: Entity<bool>,

    settings: Entity<Settings>,
    settings_visible: bool,
    previous_focus: Option<FocusHandle>,

    _sidebar_visibility_subscription: Subscription,
    _settings_subscription: Subscription,
}

impl Layout {
    fn new(
        collections: Arc<CollectionRegistry>,
        updater: Entity<Updater>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let sidebar_visible = cx.new(|_| true);
        let bottom_panel = cx.new(|cx| BottomPanel::new(sidebar_visible.clone(), cx));
        let sidebar_visibility_subscription = cx.observe(&sidebar_visible, |_, _, cx| cx.notify());
        let settings = cx.new(|cx| Settings::new(updater, window, cx));
        let settings_subscription = cx.subscribe_in(
            &settings,
            window,
            |this, _, _: &SettingsEvent, window, cx| this.close_settings(window, cx),
        );

        Self {
            top_panel: cx.new(|_| TopPanel),
            sidebar: cx.new(|_| Sidebar::new(collections)),
            main_view: cx.new(|_| MainView),
            bottom_panel,
            main_split: cx.new(|_| ResizableState::default()),
            sidebar_visible,
            settings,
            settings_visible: false,
            previous_focus: None,
            _sidebar_visibility_subscription: sidebar_visibility_subscription,
            _settings_subscription: settings_subscription,
        }
    }

    fn open_settings(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if !self.settings_visible {
            self.previous_focus = window.focused(cx);
            self.settings_visible = true;
        }

        self.settings
            .update(cx, |settings, cx| settings.focus(window, cx));

        cx.notify();
    }

    fn close_settings(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if !self.settings_visible {
            return;
        }

        self.settings_visible = false;

        if let Some(focus) = self.previous_focus.take() {
            window.focus(&focus, cx);
        } else {
            window.blur(cx);
        }

        cx.notify();
    }

    fn toggle_sidebar(&mut self, cx: &mut Context<Self>) {
        self.sidebar_visible.update(cx, |visible, cx| {
            *visible = !*visible;

            cx.notify();
        });
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

                    this.settings.update(cx, |settings, cx| {
                        settings.select_page(SettingsPage::General, window, cx)
                    });
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
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let sidebar_progress = motion::transition(
            "sidebar-visibility",
            if *self.sidebar_visible.read(cx) {
                1.0
            } else {
                0.0
            },
            Transition::new(Duration::from_millis(150)).ease(ease_in_out_cubic),
            window,
            cx,
        );

        let sidebar_width = self
            .main_split
            .read(cx)
            .sizes()
            .first()
            .copied()
            .unwrap_or(px(200.))
            .clamp(px(200.), px(400.));

        let workspace = v_flex()
            .size_full()
            .when(self.settings_visible, |this| this.hidden())
            .child(self.top_panel.clone())
            .child(
                div().flex_1().min_h_0().overflow_hidden().child(
                    h_resizable("main_split")
                        .with_state(&self.main_split)
                        .child(
                            resizable_panel()
                                .visible(sidebar_progress > 0.0)
                                .flex_none()
                                .ml(sidebar_width * (sidebar_progress - 1.0))
                                .size_range(px(200.)..px(400.))
                                .child(self.sidebar.clone()),
                        )
                        .child(self.main_view.clone().into_any_element()),
                ),
            )
            .child(self.bottom_panel.clone());

        div().size_full().child(workspace).child(
            div()
                .size_full()
                .when(!self.settings_visible, |this| this.hidden())
                .child(self.settings.clone()),
        )
    }
}

pub fn init(collections: CollectionRegistry, updater: Entity<Updater>, cx: &mut App) {
    crate::actions::init(cx);

    let window_options = crate::window_options::use_window_options(cx);
    cx.open_window(window_options, move |window, cx| {
        let layout = cx.new(|cx| Layout::new(Arc::new(collections), updater, window, cx));
        on_toggle_sidebar(&layout, cx);
        on_open_settings(&layout, window.window_handle(), cx);

        cx.new(|cx| Root::new(layout.clone(), window, cx).bg(cx.theme().background))
    })
    .expect("Failed to open the window");
}

#[cfg(test)]
mod tests {
    use super::{Layout, on_toggle_sidebar};
    use crate::actions::{CloseSettings, ToggleLeftSidebar};
    use crate::layout::bottom_panel::TOGGLE_SIDEBAR_BUTTON;
    use collection::CollectionRegistry;
    use gpui_kit::{Modifiers, TestAppContext, px};
    use std::sync::Arc;

    #[gpui_kit::test]
    fn settings_survives_closing_and_reopening(cx: &mut TestAppContext) {
        cx.update(|cx| {
            gpui_kit::init(cx);
            crate::actions::init(cx);
        });

        let (layout, cx) = cx.add_window_view(|window, cx| {
            Layout::new(
                Arc::new(CollectionRegistry::new()),
                updater::init("1.2.3", cx),
                window,
                cx,
            )
        });

        let settings = cx.read(|cx| layout.read(cx).settings.clone());
        assert!(cx.debug_bounds("settings").is_none());
        assert!(cx.debug_bounds("main-view").is_some());
        cx.update(|window, cx| assert!(window.focused(cx).is_none()));

        for _ in 0..2 {
            cx.update(|window, cx| {
                layout.update(cx, |layout, cx| layout.open_settings(window, cx));
            });
            cx.run_until_parked();

            assert!(cx.debug_bounds("settings").is_some());
            assert!(cx.debug_bounds("main-view").is_none());
            cx.read(|cx| {
                assert!(layout.read(cx).settings_visible);
                assert_eq!(layout.read(cx).settings, settings);
            });

            // Reopening an already visible screen must not replace the saved focus.
            cx.update(|window, cx| {
                layout.update(cx, |layout, cx| layout.open_settings(window, cx));
                window.dispatch_action(Box::new(CloseSettings), cx);
            });
            cx.run_until_parked();

            cx.read(|cx| {
                assert!(!layout.read(cx).settings_visible);
                assert_eq!(layout.read(cx).settings, settings);
            });
            assert!(cx.debug_bounds("settings").is_none());
            assert!(cx.debug_bounds("main-view").is_some());
            cx.update(|window, cx| assert!(window.focused(cx).is_none()));
        }
    }

    #[gpui_kit::test]
    fn toggle_sidebar_action(cx: &mut TestAppContext) {
        cx.update(|cx| {
            gpui_kit::init(cx);
            cx.set_reduce_motion(true);
            crate::actions::init(cx);
        });

        let (layout, cx) = cx.add_window_view(|window, cx| {
            Layout::new(
                Arc::new(CollectionRegistry::new()),
                updater::init("1.2.3", cx),
                window,
                cx,
            )
        });
        cx.update(|_, cx| on_toggle_sidebar(&layout, cx));

        let sidebar_visible =
            |cx: &TestAppContext| cx.read(|cx| *layout.read(cx).sidebar_visible.read(cx));

        assert!(sidebar_visible(cx));

        let main_split = cx.read(|cx| layout.read(cx).main_split.clone());
        cx.update(|window, cx| {
            main_split.update(cx, |split, cx| split.resize_panel(0, px(300.), window, cx));
        });
        cx.run_until_parked();

        let panel_sizes = cx.read(|cx| main_split.read(cx).sizes().clone());
        assert_eq!(panel_sizes[0], px(300.));

        let main_bounds = cx
            .debug_bounds("main-view")
            .expect("main view should be rendered");

        // The tooltip hint shows this binding.
        cx.update(|window, _| {
            let binding = window
                .highest_precedence_binding_for_action(&ToggleLeftSidebar)
                .expect("ToggleLeftSidebar should be bound");
            assert_eq!(binding.keystrokes()[0].inner().to_string(), "⌘B");
        });

        cx.simulate_keystrokes("cmd-b");
        assert!(!sidebar_visible(cx));

        let expanded_bounds = cx
            .debug_bounds("main-view")
            .expect("main view should remain rendered");
        assert_eq!(
            expanded_bounds.size.width,
            main_bounds.size.width + px(300.)
        );

        cx.simulate_keystrokes("cmd-b");
        assert!(sidebar_visible(cx));

        assert_eq!(cx.debug_bounds("main-view"), Some(main_bounds));

        assert_eq!(
            cx.read(|cx| main_split.read(cx).sizes().clone()),
            panel_sizes
        );

        let button_bounds = cx
            .debug_bounds(TOGGLE_SIDEBAR_BUTTON)
            .expect("toggle-sidebar button should be rendered");
        cx.simulate_click(button_bounds.center(), Modifiers::default());
        assert!(!sidebar_visible(cx));
    }
}
