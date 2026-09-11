use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AppearanceMode {
    System,
    Light,
    #[default]
    Dark,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(default)]
pub struct AppearancePreferences {
    pub mode: AppearanceMode,
    pub light_theme: String,
    pub dark_theme: String,
    pub editor_font: String,
    pub interface_font_size: f32,
}

impl Default for AppearancePreferences {
    fn default() -> Self {
        Self {
            mode: AppearanceMode::Dark,
            light_theme: "Ayu Light".into(),
            dark_theme: "Ayu Dark".into(),
            editor_font: String::new(),
            interface_font_size: 16.,
        }
    }
}

impl AppearancePreferences {
    pub(crate) fn normalize(&mut self) {
        self.interface_font_size = if self.interface_font_size.is_finite() {
            self.interface_font_size.clamp(12., 24.)
        } else {
            16.
        };
        self.editor_font = self.editor_font.trim().chars().take(200).collect();
    }
}
