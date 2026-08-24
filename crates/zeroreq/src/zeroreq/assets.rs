use std::borrow::Cow;

use gpui::{AssetSource, Result, SharedString};

/// Serves Zeroreq's own icons, falling back to the icons bundled with
/// gpui-component.
pub struct Assets;

const LAYOUT_SIDEBAR_FILLED: &[u8] = include_bytes!("../../assets/icons/layout-sidebar-filled.svg");
const LAYOUT_SIDEBAR_INACTIVE: &[u8] =
    include_bytes!("../../assets/icons/layout-sidebar-inactive.svg");

const LOCAL_ICONS: [(&str, &[u8]); 2] = [
    ("icons/layout-sidebar-filled.svg", LAYOUT_SIDEBAR_FILLED),
    ("icons/layout-sidebar-inactive.svg", LAYOUT_SIDEBAR_INACTIVE),
];

impl AssetSource for Assets {
    fn load(&self, path: &str) -> Result<Option<Cow<'static, [u8]>>> {
        if let Some((_, bytes)) = LOCAL_ICONS.iter().find(|(icon_path, _)| *icon_path == path) {
            return Ok(Some(Cow::Borrowed(*bytes)));
        }
        gpui_component_assets::Assets.load(path)
    }

    fn list(&self, path: &str) -> Result<Vec<SharedString>> {
        let mut paths = gpui_component_assets::Assets.list(path)?;
        paths.extend(
            LOCAL_ICONS
                .iter()
                .map(|(icon_path, _)| *icon_path)
                .filter(|icon_path| icon_path.starts_with(path))
                .map(SharedString::from),
        );
        Ok(paths)
    }
}
