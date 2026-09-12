/// Each bundled family has one light and one dark palette.
pub const THEME_PAIRS: &[(&str, &str)] = &[
    ("Default Light", "Default Dark"),
    ("Ayu Light", "Ayu Dark"),
    ("Catppuccin Latte", "Catppuccin Mocha"),
    ("Everforest Light", "Everforest Dark"),
    ("Flexoki Light", "Flexoki Dark"),
    ("Gruvbox Light", "Gruvbox Dark"),
    ("Hybrid Light", "Hybrid Dark"),
    ("macOS Classic Light", "macOS Classic Dark"),
    ("Mellifluous Light", "Mellifluous Dark"),
    ("Molokai Light", "Molokai Dark"),
    ("Solarized Light", "Solarized Dark"),
];

pub fn theme_pair(name: &str) -> Option<(&'static str, &'static str)> {
    THEME_PAIRS
        .iter()
        .copied()
        .find(|(light, dark)| name == *light || name == *dark)
}
