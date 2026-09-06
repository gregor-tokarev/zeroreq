mod keybindings;

use gpui::{prelude::FluentBuilder as _, *};
use gpui_component::{
    button::*,
    resizable::{h_resizable, resizable_panel},
    scroll::ScrollableElement as _,
    *,
};

use crate::actions::CloseSettings;
use keybindings::KeybindingsPage;

pub(crate) enum SettingsEvent {
    Close,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum SettingsPage {
    General,
    Keybindings,
}

impl SettingsPage {
    fn title(self) -> &'static str {
        match self {
            Self::General => "General",
            Self::Keybindings => "Keybindings",
        }
    }

    fn icon(self) -> Icon {
        match self {
            Self::General => Icon::new(IconName::Settings2),
            Self::Keybindings => Icon::default().path("icons/keyboard.svg"),
        }
    }
}

pub(crate) struct Settings {
    page: SettingsPage,
    general: AnyView,
    keybindings: Entity<KeybindingsPage>,
    focus_handle: FocusHandle,
}

impl EventEmitter<SettingsEvent> for Settings {}

impl Settings {
    pub(crate) fn new(general: AnyView, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let this = Self {
            page: SettingsPage::General,
            general,
            keybindings: cx.new(|cx| KeybindingsPage::new(window, cx)),
            focus_handle: cx.focus_handle(),
        };

        this.focus(window, cx);

        this
    }

    pub(crate) fn focus(&self, window: &mut Window, cx: &mut Context<Self>) {
        match self.page {
            SettingsPage::General => window.focus(&self.focus_handle, cx),
            SettingsPage::Keybindings => self
                .keybindings
                .update(cx, |page, cx| page.focus_search(window, cx)),
        }
    }

    pub(crate) fn select_page(
        &mut self,
        page: SettingsPage,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.page = page;
        self.focus(window, cx);

        cx.notify();
    }

    fn close(&mut self, _: &CloseSettings, _: &mut Window, cx: &mut Context<Self>) {
        cx.emit(SettingsEvent::Close);
    }

    fn page_buttons(&self, cx: &mut Context<Self>) -> Vec<Button> {
        [SettingsPage::General, SettingsPage::Keybindings]
            .into_iter()
            .map(|page| {
                Button::new(page.title())
                    .ghost()
                    .h(px(44.))
                    .px_3()
                    .rounded_lg()
                    .text_size(px(16.))
                    .text_color(cx.theme().muted_foreground)
                    .when(self.page == page, |this| {
                        this.bg(cx.theme().primary.opacity(0.16))
                            .text_color(cx.theme().foreground)
                    })
                    .icon(page.icon().size_5())
                    .label(page.title())
                    .child(div().flex_1())
                    .on_click(
                        cx.listener(move |this, _, window, cx| this.select_page(page, window, cx)),
                    )
            })
            .collect()
    }

    fn render_sidebar(&self, cx: &mut Context<Self>) -> impl IntoElement + use<> {
        v_flex()
            .w_full()
            .h_full()
            .border_r_1()
            .border_color(cx.theme().border)
            .bg(cx.theme().sidebar)
            .pt(px(48.))
            .px_3()
            .pb_3()
            .children(
                self.page_buttons(cx)
                    .into_iter()
                    .map(|button| button.w_full()),
            )
            .child(div().flex_1())
            .child(
                Button::new("settings-back")
                    .ghost()
                    .h(px(44.))
                    .w_full()
                    .px_3()
                    .rounded_lg()
                    .text_size(px(15.))
                    .text_color(cx.theme().muted_foreground)
                    .icon(IconName::ArrowLeft)
                    .label("Back to workspace")
                    .child(div().flex_1())
                    .on_click(
                        cx.listener(|this, _, window, cx| this.close(&CloseSettings, window, cx)),
                    ),
            )
    }
}

impl Render for Settings {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let narrow = window.viewport_size().width < px(680.);

        let content = v_flex()
            .flex_1()
            .min_w_0()
            .h_full()
            .child(
                h_flex()
                    .h(px(44.))
                    .flex_none()
                    .px_5()
                    .gap_2()
                    .border_b_1()
                    .border_color(cx.theme().border)
                    .when(narrow, |this| this.pl_20())
                    .child(
                        div()
                            .text_xs()
                            .text_color(cx.theme().muted_foreground)
                            .child("Settings"),
                    )
                    .child(
                        Icon::new(IconName::ChevronRight)
                            .size_3()
                            .text_color(cx.theme().muted_foreground),
                    )
                    .child(div().text_xs().child(self.page.title()))
                    .child(div().flex_1())
                    .child(
                        Button::new("close-settings")
                            .ghost()
                            .small()
                            .icon(IconName::Close)
                            .tooltip_with_action("Close settings", &CloseSettings, Some("Settings"))
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.close(&CloseSettings, window, cx)
                            })),
                    ),
            )
            .when(narrow, |this| {
                this.child(
                    h_flex()
                        .px_3()
                        .py_2()
                        .gap_2()
                        .children(self.page_buttons(cx)),
                )
            })
            .child(
                div()
                    .id("settings-scroll")
                    .flex_1()
                    .min_h_0()
                    .overflow_y_scrollbar()
                    .child(
                        v_flex()
                            .items_center()
                            .px(if narrow { px(20.) } else { px(48.) })
                            .py(if narrow { px(24.) } else { px(48.) })
                            .child(match self.page {
                                SettingsPage::General => self.general.clone().into_any_element(),
                                SettingsPage::Keybindings => {
                                    self.keybindings.clone().into_any_element()
                                }
                            }),
                    ),
            );

        h_flex()
            .id("settings")
            .debug_selector(|| "settings".into())
            .key_context("Settings")
            .track_focus(&self.focus_handle)
            .on_action(cx.listener(Self::close))
            .relative()
            .size_full()
            .overflow_hidden()
            .text_sm()
            .text_color(cx.theme().foreground)
            .bg(cx.theme().background)
            .child(if narrow {
                content.into_any_element()
            } else {
                h_resizable("settings-panels")
                    .child(
                        resizable_panel()
                            .size(px(232.))
                            .size_range(
                                px(220.)..px(400.).min(window.viewport_size().width - px(448.)),
                            )
                            .flex_none()
                            .child(self.render_sidebar(cx)),
                    )
                    .child(
                        resizable_panel()
                            .size_range(px(448.)..Pixels::MAX)
                            .child(content),
                    )
                    .into_any_element()
            })
    }
}
