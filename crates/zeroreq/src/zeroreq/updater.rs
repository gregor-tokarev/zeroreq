mod install;

#[cfg(test)]
mod tests;

use std::sync::Arc;

use gpui::{
    App, AppContext, Context, Entity,
    http_client::{AsyncBody, HttpClient},
};
use semver::Version;
use serde::Deserialize;
use smol::io::AsyncReadExt;

const UPDATE_MANIFEST_URL: &str =
    "https://github.com/gregor-tokarev/zeroreq/releases/latest/download/zeroreq-update.json";

pub const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Clone, Debug, Deserialize)]
pub struct UpdateManifest {
    pub version: String,
    url: String,
    sha256: String,
}

#[derive(Clone, Debug)]
pub enum UpdateStatus {
    Idle,
    Checking,
    UpToDate,
    Available(UpdateManifest),
    Installing(String),
    Error(String),
}

pub struct Updater {
    status: UpdateStatus,
}

impl Updater {
    pub fn status(&self) -> &UpdateStatus {
        &self.status
    }

    fn set_status(&mut self, status: UpdateStatus, cx: &mut Context<Self>) {
        self.status = status;

        cx.notify();
    }

    pub fn check(&mut self, cx: &mut Context<Self>) {
        if matches!(
            self.status,
            UpdateStatus::Checking | UpdateStatus::Installing(_)
        ) {
            return;
        }

        self.set_status(UpdateStatus::Checking, cx);

        let http_client = cx.http_client();

        cx.spawn(async move |this, cx| {
            let status = match check_for_update(http_client).await {
                Ok(Some(manifest)) => UpdateStatus::Available(manifest),
                Ok(None) => UpdateStatus::UpToDate,
                Err(error) => UpdateStatus::Error(error),
            };

            let _ = this.update(cx, |this, cx| this.set_status(status, cx));
        })
        .detach();
    }

    pub fn install(&mut self, cx: &mut Context<Self>) {
        let UpdateStatus::Available(manifest) = self.status.clone() else {
            return;
        };

        self.set_status(UpdateStatus::Installing(manifest.version.clone()), cx);

        let http_client = cx.http_client();
        let task = cx.background_executor().spawn(async move {
            install::download_and_prepare_update(&manifest, http_client).await
        });

        cx.spawn(async move |this, cx| match task.await {
            Ok(()) => cx.update(|cx| cx.quit()),
            Err(error) => {
                let _ = this.update(cx, |this, cx| {
                    this.set_status(UpdateStatus::Error(error), cx)
                });
            }
        })
        .detach();
    }
}

pub fn init(cx: &mut App) -> Entity<Updater> {
    let updater = cx.new(|_| Updater {
        status: UpdateStatus::Idle,
    });

    // Bare cargo binaries cannot be replaced by the app-bundle installer.
    if install::current_app_bundle().is_ok() {
        updater.update(cx, |updater, cx| updater.check(cx));
    }

    updater
}

async fn check_for_update(
    http_client: Arc<dyn HttpClient>,
) -> Result<Option<UpdateManifest>, String> {
    let mut response = http_client
        .get(UPDATE_MANIFEST_URL, AsyncBody::empty(), true)
        .await
        .map_err(|error| format!("Could not check for updates: {error}"))?;
    let status = response.status();

    let mut body = Vec::new();
    response
        .body_mut()
        .read_to_end(&mut body)
        .await
        .map_err(|error| format!("Could not read the update manifest: {error}"))?;

    if !status.is_success() {
        let detail = String::from_utf8_lossy(&body).trim().to_string();

        return Err(if detail.is_empty() {
            format!("GitHub returned {status} for the update manifest.")
        } else {
            detail
        });
    }

    let manifest: UpdateManifest = serde_json::from_slice(&body)
        .map_err(|error| format!("The update manifest is invalid: {error}"))?;

    let installed = Version::parse(CURRENT_VERSION)
        .map_err(|error| format!("The installed version is invalid: {error}"))?;
    let released = Version::parse(manifest.version.trim_start_matches('v'))
        .map_err(|error| format!("The released version is invalid: {error}"))?;

    Ok((released > installed).then_some(manifest))
}
