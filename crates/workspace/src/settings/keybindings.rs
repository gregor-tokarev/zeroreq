mod page;
mod recorder;

use page::shortcut_keycaps;

#[cfg(test)]
use page::matches_search;

#[cfg(test)]
mod tests;

pub(super) use page::KeybindingsPage;
