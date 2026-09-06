use std::rc::Rc;

use gpui::{
    Action, DummyKeyboardMapper, InvalidKeystrokeError, KeyBinding, KeyBindingContextPredicate,
    Keystroke, SharedString, Unbind,
};

use crate::KeybindingError;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Binding {
    pub keystrokes: String,
    pub context: Option<String>,
}

pub(super) struct RegisteredBinding {
    pub(super) action_name: &'static str,
    pub(super) binding: Binding,
}

impl Binding {
    pub(super) fn new(keystrokes: &str, context: Option<&str>) -> Result<Self, KeybindingError> {
        let keystrokes = keystrokes.trim();
        if keystrokes.is_empty() {
            return Err(KeybindingError::EmptyKeystrokes);
        }

        if let Some(context) = context {
            KeyBindingContextPredicate::parse(context).map_err(|source| {
                KeybindingError::InvalidContext {
                    context: context.to_owned(),
                    source,
                }
            })?;
        }

        Ok(Self {
            keystrokes: keystrokes
                .split_whitespace()
                .map(Keystroke::parse)
                .map(|stroke| stroke.map(|stroke| stroke.unparse()))
                .collect::<Result<Vec<_>, _>>()?
                .join(" "),
            context: context.map(str::to_owned),
        })
    }

    pub(super) fn to_gpui(
        &self,
        action: Box<dyn Action>,
    ) -> Result<KeyBinding, InvalidKeystrokeError> {
        let predicate = self
            .context
            .as_deref()
            .map(KeyBindingContextPredicate::parse)
            .transpose()
            .expect("binding context was validated when the binding was created")
            .map(Rc::new);

        KeyBinding::load(
            &self.keystrokes,
            action,
            predicate,
            false,
            None,
            &DummyKeyboardMapper,
        )
    }
}

impl RegisteredBinding {
    pub(super) fn unbind(self) -> KeyBinding {
        KeyBinding::new(
            &self.binding.keystrokes,
            Unbind(SharedString::from(self.action_name)),
            self.binding.context.as_deref(),
        )
    }
}
