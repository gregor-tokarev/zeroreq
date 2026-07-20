use gpui::*;
use gpui_component::{button::*, *};
use std::sync::Arc;

mod zeroreq;

struct HelloWorld;

impl Render for HelloWorld {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .v_flex()
            .size_full()
            .bg(cx.theme().background)
            .text_color(cx.theme().foreground)
            .child(div().h(px(38.)).w_full().flex_none())
            .child(
                div()
                    .v_flex()
                    .gap_2()
                    .flex_1()
                    .items_center()
                    .justify_center()
                    .child("Hello, World!")
                    .child(
                        Button::new("ok")
                            .primary()
                            .label("Let's Go!")
                            .on_click(|_, _, _| println!("Clicked!")),
                    ),
            )
    }
}

fn main() {
    let user_agent = format!(
        "Zeroreq/{} ({}; {})",
        env!("CARGO_PKG_VERSION"),
        std::env::consts::OS,
        std::env::consts::ARCH
    );
    let http_client = reqwest_client::ReqwestClient::user_agent(&user_agent)
        .expect("Failed to initialize the HTTP client");
    let app = gpui_platform::application().with_http_client(Arc::new(http_client));

    app.run(move |cx: &mut App| {
        gpui_component::init(cx);

        zeroreq_theme::init(cx);
        zeroreq::actions::init(cx);
        zeroreq::updater::start_automatic_check(cx);

        let menus = zeroreq::menu::use_menus(cx);
        cx.set_menus(menus);

        let window_options = zeroreq::window_options::use_window_options(cx);

        cx.spawn(async move |cx| {
            cx.open_window(window_options, |window, cx| {
                let view = cx.new(|_| HelloWorld);
                cx.new(|cx| Root::new(view, window, cx).bg(cx.theme().background))
            })
            .expect("Failed to open the window")
        })
        .detach();
    });
}
