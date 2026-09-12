//! Theme setup shared by Request Eagle windows.

mod appearance;
mod catalog;
mod registry;

#[cfg(test)]
mod tests;

pub use appearance::apply_preferences;
pub use catalog::{THEME_PAIRS, theme_pair};
pub use registry::{apply, init};
