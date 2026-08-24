use gpui::*;
use gpui_component::{ActiveTheme as _, Colorize as _, IconNamed, Sizable as _, button::*};

use crate::actions::ToggleLeftSidebar;

pub(crate) const TOGGLE_SIDEBAR_BUTTON: &str = "toggle-sidebar";

enum SidebarIcon {
    Visible,
    Hidden,
}

impl IconNamed for SidebarIcon {
    fn path(self) -> SharedString {
        // Tabler icons: `layout-sidebar` (filled) and `layout-sidebar-inactive`.
        match self {
            Self::Visible => "icons/layout-sidebar-filled.svg".into(),
            Self::Hidden => "icons/layout-sidebar-inactive.svg".into(),
        }
    }
}

#[derive(IntoElement)]
pub struct BottomPanel {
    sidebar_visible: bool,
}

impl BottomPanel {
    pub fn new(sidebar_visible: bool) -> Self {
        Self { sidebar_visible }
    }
}

impl RenderOnce for BottomPanel {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let icon = if self.sidebar_visible {
            SidebarIcon::Visible
        } else {
            SidebarIcon::Hidden
        };

        div()
            .border_t_1()
            .border_color(cx.theme().border)
            .flex()
            .flex_none()
            .items_center()
            .h_8()
            .px_2()
            .bg(cx.theme().background.darken(0.20))
            .child(
                div()
                    .flex_none()
                    .debug_selector(|| TOGGLE_SIDEBAR_BUTTON.to_string())
                    .child(
                        Button::new(TOGGLE_SIDEBAR_BUTTON)
                            .ghost()
                            .small()
                            .icon(icon)
                            .tooltip_with_action("Toggle Sidebar", &ToggleLeftSidebar, None)
                            .on_click(|_, window, cx| {
                                window.dispatch_action(ToggleLeftSidebar.boxed_clone(), cx);
                            }),
                    ),
            )
            .child("bottom panel")
    }
}
