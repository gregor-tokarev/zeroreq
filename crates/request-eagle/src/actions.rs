mod handlers;

#[cfg(test)]
mod tests;

pub use handlers::{CheckForUpdates, Quit, init};
