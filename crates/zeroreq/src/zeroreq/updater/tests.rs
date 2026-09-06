use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

use gpui::{
    AppContext as _, Modifiers, TestAppContext,
    http_client::{FakeHttpClient, Response},
};

use super::{CURRENT_VERSION, UPDATE_MANIFEST_URL, UpdateStatus, Updater, check_for_update};
use crate::zeroreq::{
    actions::{self, CheckForUpdates},
    general_settings::GeneralSettings,
};

fn manifest(version: &str) -> String {
    serde_json::json!({
        "version": version,
        "url": "https://example.test/Zeroreq.zip",
        "sha256": "abc123",
    })
    .to_string()
}

#[gpui::test]
fn checking_survives_closing_settings_and_does_not_open_a_window(cx: &mut TestAppContext) {
    let requests = Arc::new(AtomicUsize::new(0));
    let (respond, wait) = smol::channel::bounded::<()>(1);
    let http = FakeHttpClient::create({
        let requests = requests.clone();
        move |request| {
            assert_eq!(request.uri().to_string(), UPDATE_MANIFEST_URL);
            requests.fetch_add(1, Ordering::SeqCst);
            let wait = wait.clone();

            async move {
                wait.recv().await.unwrap();

                Ok(Response::builder()
                    .status(200)
                    .body(manifest("99.0.0").into())
                    .unwrap())
            }
        }
    });

    cx.update(|cx| {
        gpui_component::init(cx);
        cx.set_http_client(http);
    });

    let updater = cx.new(|_| Updater {
        status: UpdateStatus::Idle,
    });

    let (page, view) = cx.add_window_view(|_, cx| GeneralSettings::new(updater.clone(), cx));

    let check = view.debug_bounds("check-for-updates").unwrap();
    view.simulate_click(check.center(), Modifiers::default());

    // Both the button and another caller must coalesce onto the in-flight check.
    view.simulate_click(check.center(), Modifiers::default());
    updater.update(view, |updater, cx| updater.check(cx));
    view.run_until_parked();

    assert_eq!(requests.load(Ordering::SeqCst), 1);
    assert!(matches!(
        updater.read_with(view, |updater, _| updater.status.clone()),
        UpdateStatus::Checking
    ));

    view.update(|window, _| window.remove_window());
    drop(page);

    respond.try_send(()).unwrap();
    cx.run_until_parked();

    cx.read(|cx| {
        assert!(
            cx.windows().is_empty(),
            "A completed background check must not open UI"
        );
        assert!(matches!(
            updater.read(cx).status(),
            UpdateStatus::Available(manifest) if manifest.version == "99.0.0"
        ));
    });

    let (_, view) = cx.add_window_view(|_, cx| GeneralSettings::new(updater.clone(), cx));
    assert!(
        view.debug_bounds("install-update").is_some(),
        "Reopened General must offer the completed update"
    );
    assert_eq!(requests.load(Ordering::SeqCst), 1);
}

#[gpui::test]
fn failed_check_can_be_retried_from_general(cx: &mut TestAppContext) {
    let requests = Arc::new(AtomicUsize::new(0));
    let http = FakeHttpClient::create({
        let requests = requests.clone();
        move |_| {
            let attempt = requests.fetch_add(1, Ordering::SeqCst);

            async move {
                Ok(if attempt == 0 {
                    Response::builder()
                        .status(503)
                        .body("Release service unavailable".into())
                        .unwrap()
                } else {
                    Response::builder()
                        .status(200)
                        .body(manifest("99.0.0").into())
                        .unwrap()
                })
            }
        }
    });

    cx.update(|cx| {
        gpui_component::init(cx);
        cx.set_http_client(http);
    });

    let updater = cx.new(|_| Updater {
        status: UpdateStatus::Idle,
    });

    let (_, view) = cx.add_window_view(|_, cx| GeneralSettings::new(updater.clone(), cx));

    let check = view.debug_bounds("check-for-updates").unwrap();
    view.simulate_click(check.center(), Modifiers::default());

    view.read(|cx| {
        assert!(matches!(
            updater.read(cx).status(),
            UpdateStatus::Error(error) if error == "Release service unavailable"
        ));
    });

    let retry = view.debug_bounds("check-for-updates").unwrap();
    view.simulate_click(retry.center(), Modifiers::default());

    view.read(|cx| {
        assert!(matches!(
            updater.read(cx).status(),
            UpdateStatus::Available(_)
        ));
    });

    let install = view
        .debug_bounds("install-update")
        .expect("General must update its button when the check completes");

    // Cargo test binaries have no app bundle. Exercise the real install button
    // and failure state without downloading or replacing an app.
    assert!(super::install::current_app_bundle().is_err());
    view.simulate_click(install.center(), Modifiers::default());

    view.read(|cx| {
        assert!(matches!(
            updater.read(cx).status(),
            UpdateStatus::Error(error) if error.contains("only available from Zeroreq.app")
        ));
    });

    assert!(view.debug_bounds("check-for-updates").is_some());
    assert_eq!(requests.load(Ordering::SeqCst), 2);
}

#[gpui::test]
fn menu_opens_general_but_cannot_interrupt_installation(cx: &mut TestAppContext) {
    let requests = Arc::new(AtomicUsize::new(0));
    let http = FakeHttpClient::create({
        let requests = requests.clone();
        move |_| {
            requests.fetch_add(1, Ordering::SeqCst);
            async {
                Ok(Response::builder()
                    .status(200)
                    .body(manifest("99.0.0").into())
                    .unwrap())
            }
        }
    });

    let opened = Arc::new(AtomicUsize::new(0));
    let updater = cx.new(|_| Updater {
        status: UpdateStatus::Idle,
    });

    cx.update(|cx| {
        gpui_component::init(cx);
        cx.set_http_client(http);
        actions::init(updater.clone(), cx);

        let opened = opened.clone();
        cx.on_action(move |_: &workspace::OpenGeneralSettings, _| {
            opened.fetch_add(1, Ordering::SeqCst);
        });
    });

    let (_, view) = cx.add_window_view(|_, _| gpui::Empty);
    view.update(|window, _| window.activate_window());

    view.dispatch_action(CheckForUpdates);

    assert_eq!(opened.load(Ordering::SeqCst), 1);
    assert_eq!(requests.load(Ordering::SeqCst), 1);
    view.read(|cx| {
        assert!(matches!(
            updater.read(cx).status(),
            UpdateStatus::Available(_)
        ))
    });

    // An installation is already in progress. Menu actions must keep opening
    // its status without starting another check or installer.
    updater.update(view, |updater, cx| {
        updater.set_status(UpdateStatus::Installing("99.0.0".into()), cx)
    });
    view.dispatch_action(CheckForUpdates);
    updater.update(view, |updater, cx| updater.install(cx));
    view.run_until_parked();

    assert_eq!(opened.load(Ordering::SeqCst), 2);
    assert_eq!(requests.load(Ordering::SeqCst), 1);
    view.read(|cx| {
        assert!(matches!(
            updater.read(cx).status(),
            UpdateStatus::Installing(version) if version == "99.0.0"
        ));
    });
}

#[gpui::test]
async fn update_check_handles_version_boundaries_and_invalid_responses() {
    for (version, available) in [
        ("99.0.0", true),
        ("v99.0.0", true),
        (CURRENT_VERSION, false),
        ("0.0.1", false),
    ] {
        let http = FakeHttpClient::create(move |_| async move {
            Ok(Response::builder()
                .status(200)
                .body(manifest(version).into())
                .unwrap())
        });

        assert_eq!(
            check_for_update(http).await.unwrap().is_some(),
            available,
            "release {version}"
        );
    }

    for body in ["not json".to_owned(), manifest("invalid-version")] {
        let http = FakeHttpClient::create(move |_| {
            let body = body.clone();
            async { Ok(Response::builder().status(200).body(body.into()).unwrap()) }
        });

        assert!(check_for_update(http).await.is_err());
    }
}
