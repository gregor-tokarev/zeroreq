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
