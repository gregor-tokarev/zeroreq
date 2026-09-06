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

    let app = gpui_platform::application()
        .with_assets(zeroreq::assets::Assets)
        .with_http_client(Arc::new(http_client));

    app.run(move |cx: &mut App| {
        gpui_component::init(cx);
        keybindings_service::init(cx);

        if let Some(home) = std::env::home_dir()
            && let Err(error) =
                keybindings_service::load_overrides(home.join(".zeroreq/keybindings.json"), cx)
        {
            eprintln!("Failed to load key bindings: {error}");
        }

        zeroreq_theme::init(cx);

        let updater = zeroreq::updater::init(cx);
        zeroreq::actions::init(updater.clone(), cx);

        let general_settings =
            cx.new(|cx| zeroreq::general_settings::GeneralSettings::new(updater, cx));

        let collections = collection::CollectionRegistry::load()
            .expect("Failed to load collections from ~/.zeroreq/collections");

        workspace::init(collections, general_settings.into(), cx);
        zeroreq::menu::init(cx);
    });
}
