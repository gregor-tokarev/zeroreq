use std::{
    env, fs,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::Arc,
};

use gpui::{
    App, AppContext, Context, IntoElement, ParentElement, Render, Size, Styled, TitlebarOptions,
    Window, WindowBounds, WindowKind, WindowOptions, div,
    http_client::{AsyncBody, HttpClient},
    px,
};
use gpui_component::{ActiveTheme as _, StyledExt, button::*};
use semver::Version;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use smol::{
    fs::File,
    io::{AsyncReadExt, AsyncWriteExt},
};

const UPDATE_MANIFEST_URL: &str =
    "https://github.com/gregor-tokarev/zeroreq/releases/latest/download/zeroreq-update.json";
const EXPECTED_TEAM_ID: &str = "P2M3JQ4DR5";
const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Clone, Debug, Deserialize)]
struct UpdateManifest {
    version: String,
    url: String,
    sha256: String,
}

#[derive(Clone, Debug)]
enum UpdateStatus {
    Checking,
    UpToDate,
    Available(UpdateManifest),
    Installing(String),
    Error(String),
}

struct UpdateWindow {
    status: UpdateStatus,
}

impl UpdateWindow {
    fn new(manifest: Option<UpdateManifest>, cx: &mut Context<Self>) -> Self {
        match manifest {
            Some(manifest) => Self {
                status: UpdateStatus::Available(manifest),
            },
            None => {
                let mut this = Self {
                    status: UpdateStatus::Checking,
                };
                this.check(cx);
                this
            }
        }
    }

    fn set_status(&mut self, status: UpdateStatus, cx: &mut Context<Self>) {
        self.status = status;
        cx.notify();
    }

    fn check(&mut self, cx: &mut Context<Self>) {
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

    fn install(&mut self, manifest: UpdateManifest, cx: &mut Context<Self>) {
        self.set_status(UpdateStatus::Installing(manifest.version.clone()), cx);

        let http_client = cx.http_client();
        let task = cx
            .background_executor()
            .spawn(async move { download_and_prepare_update(&manifest, http_client).await });
        cx.spawn(async move |this, cx| match task.await {
            Ok(()) => {
                let _ = cx.update(|cx| cx.quit());
            }
            Err(error) => {
                let _ = this.update(cx, |this, cx| {
                    this.set_status(UpdateStatus::Error(error), cx)
                });
            }
        })
        .detach();
    }
}

fn hint(text: impl IntoElement, cx: &App) -> impl IntoElement {
    div()
        .max_w(px(520.))
        .text_size(px(13.))
        .text_color(cx.theme().muted_foreground)
        .child(text)
}

impl Render for UpdateWindow {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let content = div()
            .v_flex()
            .size_full()
            .p_8()
            .bg(cx.theme().background)
            .text_color(cx.theme().foreground)
            .items_center()
            .justify_center()
            .gap_4()
            .child(div().text_size(px(24.)).child("Zeroreq Updates"));

        match self.status.clone() {
            UpdateStatus::Checking => content.child("Checking for updates…"),
            UpdateStatus::UpToDate => content
                .child(format!("Zeroreq {CURRENT_VERSION} is the latest version."))
                .child(
                    Button::new("check-again")
                        .label("Check Again")
                        .on_click(cx.listener(|this, _, _, cx| this.check(cx))),
                ),
            UpdateStatus::Available(manifest) => content
                .child(format!("Zeroreq {} is available.", manifest.version))
                .child(hint(format!("Installed version: {CURRENT_VERSION}"), cx))
                .child(
                    Button::new("install-update")
                        .primary()
                        .label("Install and Relaunch")
                        .on_click(
                            cx.listener(move |this, _, _, cx| this.install(manifest.clone(), cx)),
                        ),
                ),
            UpdateStatus::Installing(version) => content
                .child(format!("Downloading and verifying Zeroreq {version}…"))
                .child(hint(
                    "Zeroreq will relaunch when the update is installed.",
                    cx,
                )),
            UpdateStatus::Error(error) => content
                .child("Zeroreq could not update.")
                .child(hint(error, cx))
                .child(
                    Button::new("retry-update")
                        .label("Try Again")
                        .on_click(cx.listener(|this, _, _, cx| this.check(cx))),
                ),
        }
    }
}

pub fn open_update_window(cx: &mut App) {
    open_update_window_with_manifest(None, cx);
}

fn open_update_window_with_manifest(manifest: Option<UpdateManifest>, cx: &mut App) {
    if let Some(existing) = cx
        .windows()
        .into_iter()
        .find_map(|window| window.downcast::<UpdateWindow>())
    {
        let _ = existing.update(cx, |_, window, _| window.activate_window());
        return;
    }

    let window_size = Size {
        width: px(560.),
        height: px(300.),
    };
    if let Err(error) = cx.open_window(
        WindowOptions {
            titlebar: Some(TitlebarOptions {
                title: Some("Zeroreq Updates".into()),
                appears_transparent: true,
                ..Default::default()
            }),
            window_bounds: Some(WindowBounds::centered(window_size, cx)),
            is_resizable: false,
            is_minimizable: false,
            kind: WindowKind::Floating,
            ..Default::default()
        },
        |_, cx| cx.new(|cx| UpdateWindow::new(manifest, cx)),
    ) {
        eprintln!("failed to open Zeroreq update window: {error}");
    }
}

pub fn start_automatic_check(cx: &mut App) {
    // Cargo-launched development binaries are intentionally excluded: replacing a
    // bare executable would invalidate the signature of an installed app bundle.
    if current_app_bundle().is_err() {
        return;
    }

    let http_client = cx.http_client();
    cx.spawn(async move |cx| {
        if let Ok(Some(manifest)) = check_for_update(http_client).await {
            cx.update(|cx| open_update_window_with_manifest(Some(manifest), cx));
        }
    })
    .detach();
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

async fn download_and_prepare_update(
    manifest: &UpdateManifest,
    http_client: Arc<dyn HttpClient>,
) -> Result<(), String> {
    let current_app = current_app_bundle()?;
    let install_dir = current_app
        .parent()
        .ok_or_else(|| "The installed app has no parent directory.".to_string())?;
    if !is_directory_writable(install_dir) {
        return Err(format!(
            "{} is not writable. Move Zeroreq to a folder owned by your user and try again.",
            install_dir.display()
        ));
    }

    let work_dir = env::temp_dir().join(format!("zeroreq-update-{}", std::process::id()));
    let _ = fs::remove_dir_all(&work_dir);
    fs::create_dir_all(&work_dir)
        .map_err(|error| format!("Could not create the update directory: {error}"))?;

    let archive = work_dir.join("Zeroreq.zip");
    download_update(&manifest.url, &archive, http_client).await?;

    verify_sha256(&archive, &manifest.sha256)?;

    let status = Command::new("/usr/bin/ditto")
        .args(["-x", "-k"])
        .arg(&archive)
        .arg(&work_dir)
        .status()
        .map_err(|error| format!("Could not extract the update: {error}"))?;
    if !status.success() {
        return Err("The update archive could not be extracted.".into());
    }

    let new_app = work_dir.join("Zeroreq.app");
    verify_apple_signature(&new_app)?;
    launch_installer(&current_app, &new_app, &work_dir)
}

async fn download_update(
    url: &str,
    destination: &Path,
    http_client: Arc<dyn HttpClient>,
) -> Result<(), String> {
    let mut response = http_client
        .get(url, AsyncBody::empty(), true)
        .await
        .map_err(|error| format!("Could not download the update: {error}"))?;
    let status = response.status();
    if !status.is_success() {
        let mut body = Vec::new();
        response
            .body_mut()
            .read_to_end(&mut body)
            .await
            .map_err(|error| format!("Could not read GitHub's error response: {error}"))?;
        let detail = String::from_utf8_lossy(&body).trim().to_string();
        return Err(if detail.is_empty() {
            format!("GitHub returned {status} for the update archive.")
        } else {
            detail
        });
    }

    let mut destination = File::create(destination)
        .await
        .map_err(|error| format!("Could not create the update archive: {error}"))?;
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let bytes_read = response
            .body_mut()
            .read(&mut buffer)
            .await
            .map_err(|error| format!("Could not read the update download: {error}"))?;
        if bytes_read == 0 {
            break;
        }
        destination
            .write_all(&buffer[..bytes_read])
            .await
            .map_err(|error| format!("Could not write the update archive: {error}"))?;
    }
    destination
        .flush()
        .await
        .map_err(|error| format!("Could not finish writing the update archive: {error}"))?;

    Ok(())
}

fn verify_sha256(path: &Path, expected: &str) -> Result<(), String> {
    let bytes = fs::read(path).map_err(|error| format!("Could not read the update: {error}"))?;
    let actual = format!("{:x}", Sha256::digest(bytes));
    if actual.eq_ignore_ascii_case(expected.trim()) {
        Ok(())
    } else {
        Err("The update checksum does not match the release manifest.".into())
    }
}

fn verify_apple_signature(app: &Path) -> Result<(), String> {
    if !app.join("Contents/MacOS/zeroreq").is_file() {
        return Err("The update does not contain a valid Zeroreq app bundle.".into());
    }

    let verify = Command::new("/usr/bin/codesign")
        .args(["--verify", "--deep", "--strict", "--verbose=2"])
        .arg(app)
        .output()
        .map_err(|error| format!("Could not verify the Apple signature: {error}"))?;
    if !verify.status.success() {
        return Err(format!(
            "Apple rejected the update signature: {}",
            String::from_utf8_lossy(&verify.stderr).trim()
        ));
    }

    let details = Command::new("/usr/bin/codesign")
        .args(["--display", "--verbose=4"])
        .arg(app)
        .output()
        .map_err(|error| format!("Could not inspect the Apple signature: {error}"))?;
    let details = String::from_utf8_lossy(&details.stderr);
    if !details.contains(&format!("TeamIdentifier={EXPECTED_TEAM_ID}"))
        || !details.contains("Authority=Developer ID Application:")
    {
        return Err("The update was not signed by the expected Apple Developer team.".into());
    }

    let gatekeeper = Command::new("/usr/sbin/spctl")
        .args(["--assess", "--type", "execute", "--verbose=2"])
        .arg(app)
        .output()
        .map_err(|error| format!("Could not ask Gatekeeper to verify the update: {error}"))?;
    if !gatekeeper.status.success() {
        return Err(format!(
            "Gatekeeper rejected the update: {}",
            String::from_utf8_lossy(&gatekeeper.stderr).trim()
        ));
    }

    Ok(())
}

fn launch_installer(current_app: &Path, new_app: &Path, work_dir: &Path) -> Result<(), String> {
    const SCRIPT: &str = r#"
pid="$1"
current_app="$2"
new_app="$3"
work_dir="$4"
backup="${current_app}.previous"

while kill -0 "$pid" 2>/dev/null; do
  sleep 0.2
done

rm -rf "$backup"
if mv "$current_app" "$backup" && mv "$new_app" "$current_app"; then
  open "$current_app"
  rm -rf "$backup" "$work_dir"
else
  rm -rf "$current_app"
  mv "$backup" "$current_app"
  open "$current_app"
fi
"#;

    Command::new("/bin/sh")
        .args(["-c", SCRIPT, "zeroreq-updater"])
        .arg(std::process::id().to_string())
        .arg(current_app)
        .arg(new_app)
        .arg(work_dir)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|error| format!("Could not start the update installer: {error}"))?;
    Ok(())
}

fn current_app_bundle() -> Result<PathBuf, String> {
    let executable =
        env::current_exe().map_err(|error| format!("Could not locate Zeroreq: {error}"))?;
    // Zeroreq.app/Contents/MacOS/zeroreq → Zeroreq.app
    let app = executable
        .ancestors()
        .nth(3)
        .ok_or_else(|| "Zeroreq is not running from an app bundle.".to_string())?;
    if app.extension().and_then(|value| value.to_str()) != Some("app") {
        return Err("Automatic updates are only available from Zeroreq.app.".into());
    }
    Ok(app.to_path_buf())
}

fn is_directory_writable(directory: &Path) -> bool {
    let probe = directory.join(format!(".zeroreq-write-test-{}", std::process::id()));
    let writable = fs::write(&probe, []).is_ok();
    let _ = fs::remove_file(&probe);
    writable
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn release_manifest_parses() {
        let manifest: UpdateManifest = serde_json::from_str(
            r#"{
                "version": "0.2.0",
                "url": "https://github.com/example/app/releases/download/v0.2.0/App.zip",
                "sha256": "abc123"
            }"#,
        )
        .unwrap();
        assert_eq!(manifest.version, "0.2.0");
        assert_eq!(manifest.sha256, "abc123");
    }
}
