use gpui::*;
use std::sync::Arc;

mod zeroreq;

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

        workspace::init(cx);
    });
}
