use std::{path::Path, sync::Arc};

use collection::{Collection, CollectionRegistry, Entry};
use gpui_kit::component::{
    ActiveTheme as _, StyledExt as _, button::*, scroll::ScrollableElement as _, v_flex,
};
use gpui_kit::*;

pub struct Sidebar {
    collections: Arc<CollectionRegistry>,
    scroll_handle: ScrollHandle,
}

impl Sidebar {
    pub fn new(collections: Arc<CollectionRegistry>) -> Self {
        Self {
            collections,
            scroll_handle: ScrollHandle::new(),
        }
    }

    fn empty_state(cx: &App) -> AnyElement {
        div()
            .v_flex()
            .size_full()
            .items_center()
            .justify_center()
            .gap_2()
            .text_size(rems(0.8125))
            .child(
                Button::new("add-new-collection")
                    .primary()
                    .label("+ Add new collection"),
            )
            .child(
                div()
                    .h_flex()
                    .gap_1()
                    .text_color(cx.theme().muted_foreground)
                    .child("or")
                    .child(
                        div()
                            .id("open-existing-collection")
                            .rounded_sm()
                            .px_1()
                            .text_decoration_1()
                            .cursor_default()
                            .hover(|style| style.text_color(cx.theme().primary_hover))
                            .child("Open existing collection"),
                    ),
            )
            .into_any_element()
    }

    fn collection(collection: &Collection) -> AnyElement {
        v_flex()
            .w_full()
            .child(
                div()
                    .px_3()
                    .py_2()
                    .font_weight(FontWeight::SEMIBOLD)
                    .child(path_name(&collection.path)),
            )
            .children(collection.entries.iter().map(|entry| Self::entry(entry, 1)))
            .into_any_element()
    }

    fn entry(entry: &Entry, depth: usize) -> AnyElement {
        let indentation = px(12. + depth as f32 * 16.);

        match entry {
            Entry::File(file) => div()
                .w_full()
                .py_1()
                .pl(indentation)
                .pr_3()
                .text_size(rems(0.8125))
                .child(file.name.clone())
                .into_any_element(),
            Entry::Directory(directory) => div()
                .v_flex()
                .w_full()
                .child(
                    div()
                        .w_full()
                        .py_1()
                        .pl(indentation)
                        .pr_3()
                        .text_size(rems(0.8125))
                        .font_weight(FontWeight::MEDIUM)
                        .child(directory.name.clone()),
                )
                .children(
                    directory
                        .entries
                        .iter()
                        .map(|entry| Self::entry(entry, depth + 1)),
                )
                .into_any_element(),
        }
    }
}

impl Render for Sidebar {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if self.collections.is_empty() {
            return Self::empty_state(cx);
        }

        div()
            .id("sidebar-scroll")
            .v_flex()
            .size_full()
            .overflow_y_scroll()
            .track_scroll(&self.scroll_handle)
            .vertical_scrollbar(&self.scroll_handle)
            .child(
                v_flex()
                    .w_full()
                    .flex_none()
                    .py_2()
                    .children(self.collections.collections().iter().map(Self::collection)),
            )
            .into_any_element()
    }
}

fn path_name(path: &Path) -> String {
    path.file_name()
        .unwrap_or(path.as_os_str())
        .to_string_lossy()
        .into_owned()
}
