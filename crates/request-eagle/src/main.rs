use gpui_kit::*;
use std::sync::Arc;

mod request_eagle;

fn main() {
    if let Some(home) = std::env::home_dir() {
        request_eagle::migrate_data_directory(&home)
            .expect("Failed to migrate saved data to ~/.request-eagle");
    }

    let user_agent = format!(
        "RequestEagle/{} ({}; {})",
        env!("CARGO_PKG_VERSION"),
        std::env::consts::OS,
        std::env::consts::ARCH
    );

    let http_client = reqwest_client::ReqwestClient::user_agent(&user_agent)
        .expect("Failed to initialize the HTTP client");

    let app = gpui_kit::application()
        .with_assets(request_eagle::assets::Assets)
        .with_http_client(Arc::new(http_client));

    app.run(move |cx: &mut App| {
        gpui_kit::init(cx);

        #[cfg(target_os = "macos")]
        cx.set_reduce_motion(
            objc2_app_kit::NSWorkspace::sharedWorkspace().accessibilityDisplayShouldReduceMotion(),
        );

        keybindings_service::init(cx);

        if let Some(home) = std::env::home_dir()
            && let Err(error) = keybindings_service::load_overrides(
                home.join(".request-eagle/keybindings.json"),
                cx,
            )
        {
            eprintln!("Failed to load key bindings: {error}");
        }

        request_eagle_theme::init(cx);

        let updater = updater::init(env!("CARGO_PKG_VERSION"), cx);
        request_eagle::actions::init(updater.clone(), cx);

        let collections = collection::CollectionRegistry::load()
            .expect("Failed to load collections from ~/.request-eagle/collections");

        workspace::init(collections, updater, cx);
        request_eagle::menu::init(cx);
    });
}
