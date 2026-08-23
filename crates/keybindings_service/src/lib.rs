use std::{any::TypeId, collections::HashMap, rc::Rc};

use gpui::{
    Action, App, BorrowAppContext, DummyKeyboardMapper, Global, InvalidKeystrokeError, KeyBinding,
    KeyBindingContextPredicate, SharedString, Unbind,
};
use thiserror::Error;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Binding {
    pub keystrokes: String,
    pub context: Option<String>,
}

#[derive(Default)]
pub struct KeybindingsService {
    bindings: HashMap<TypeId, RegisteredBinding>,
}

impl Global for KeybindingsService {}

struct RegisteredBinding {
    action_name: &'static str,
    binding: Binding,
}

pub fn init(cx: &mut App) {
    if !cx.has_global::<KeybindingsService>() {
        cx.set_global(KeybindingsService::default());
    }
}

/// Sets the active binding for an action. Calling this again replaces the
/// previous binding immediately without clearing bindings owned by GPUI or
/// other services.
pub fn set_binding<A: Action>(
    keystrokes: &str,
    action: A,
    context: Option<&str>,
    cx: &mut App,
) -> Result<(), KeybindingError> {
    let binding = Binding::new(keystrokes, context)?;
    let gpui_binding = binding.to_gpui(action)?;
    let action_type = TypeId::of::<A>();
    let action_name = A::name_for_type();

    init(cx);
    cx.update_global::<KeybindingsService, _>(|service, cx| {
        if service.binding_matches(action_type, &binding) {
            return;
        }

        let previous = service.bindings.insert(
            action_type,
            RegisteredBinding {
                action_name,
                binding,
            },
        );
        let mut changes = Vec::with_capacity(2);

        if let Some(previous) = previous {
            changes.push(previous.unbind());
        }
        changes.push(gpui_binding);
        cx.bind_keys(changes);
    });

    Ok(())
}

/// Removes the binding managed by this service for an action.
pub fn remove_binding<A: Action>(cx: &mut App) -> bool {
    if !cx.has_global::<KeybindingsService>() {
        return false;
    }

    cx.update_global::<KeybindingsService, _>(|service, cx| {
        let Some(binding) = service.bindings.remove(&TypeId::of::<A>()) else {
            return false;
        };

        cx.bind_keys([binding.unbind()]);
        true
    })
}

pub fn binding_for<A: Action>(cx: &App) -> Option<&Binding> {
    cx.try_global::<KeybindingsService>()?
        .bindings
        .get(&TypeId::of::<A>())
        .map(|registered| &registered.binding)
}

impl Binding {
    fn new(keystrokes: &str, context: Option<&str>) -> Result<Self, KeybindingError> {
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
            keystrokes: keystrokes.to_owned(),
            context: context.map(str::to_owned),
        })
    }

    fn to_gpui<A: Action>(&self, action: A) -> Result<KeyBinding, InvalidKeystrokeError> {
        let predicate = self
            .context
            .as_deref()
            .map(KeyBindingContextPredicate::parse)
            .transpose()
            .expect("binding context was validated when the binding was created")
            .map(Rc::new);

        KeyBinding::load(
            &self.keystrokes,
            Box::new(action),
            predicate,
            false,
            None,
            &DummyKeyboardMapper,
        )
    }
}

impl RegisteredBinding {
    fn unbind(self) -> KeyBinding {
        KeyBinding::new(
            &self.binding.keystrokes,
            Unbind(SharedString::from(self.action_name)),
            self.binding.context.as_deref(),
        )
    }
}

impl KeybindingsService {
    fn binding_matches(&self, action_type: TypeId, binding: &Binding) -> bool {
        self.bindings
            .get(&action_type)
            .is_some_and(|registered| registered.binding == *binding)
    }
}

#[derive(Debug, Error)]
pub enum KeybindingError {
    #[error("a keybinding must contain at least one keystroke")]
    EmptyKeystrokes,
    #[error(transparent)]
    InvalidKeystroke(#[from] InvalidKeystrokeError),
    #[error("invalid keybinding context {context:?}: {source}")]
    InvalidContext {
        context: String,
        #[source]
        source: anyhow::Error,
    },
}

#[cfg(test)]
mod tests {
    use gpui::{KeyContext, Keystroke, TestApp, actions};

    use super::*;

    actions!(keybindings_service_tests, [FirstAction, OtherAction]);

    #[test]
    fn replaces_a_binding_at_runtime() {
        let mut app = TestApp::new();

        app.update(|cx| set_binding("cmd-q", FirstAction, None, cx).unwrap());
        assert_eq!(
            actions_for("cmd-q", &app),
            vec!["keybindings_service_tests::FirstAction"]
        );

        app.update(|cx| set_binding("cmd-w", FirstAction, None, cx).unwrap());

        assert!(actions_for("cmd-q", &app).is_empty());
        assert_eq!(
            actions_for("cmd-w", &app),
            vec!["keybindings_service_tests::FirstAction"]
        );
        app.read_global::<KeybindingsService, _>(|_, cx| {
            assert_eq!(
                binding_for::<FirstAction>(cx),
                Some(&Binding {
                    keystrokes: "cmd-w".into(),
                    context: None,
                })
            );
        });
    }

    #[test]
    fn keeps_bindings_that_the_service_does_not_own() {
        let mut app = TestApp::new();

        app.update(|cx| {
            cx.bind_keys([KeyBinding::new("cmd-o", OtherAction, None)]);
            set_binding("cmd-q", FirstAction, None, cx).unwrap();
            set_binding("cmd-w", FirstAction, None, cx).unwrap();
        });

        assert_eq!(
            actions_for("cmd-o", &app),
            vec!["keybindings_service_tests::OtherAction"]
        );
    }

    #[test]
    fn invalid_replacement_leaves_the_current_binding_active() {
        let mut app = TestApp::new();

        app.update(|cx| {
            set_binding("cmd-q", FirstAction, None, cx).unwrap();
            assert!(set_binding("cmd-a-b", FirstAction, None, cx).is_err());
        });

        assert_eq!(
            actions_for("cmd-q", &app),
            vec!["keybindings_service_tests::FirstAction"]
        );
    }

    #[test]
    fn removes_a_managed_binding() {
        let mut app = TestApp::new();

        app.update(|cx| {
            set_binding("cmd-q", FirstAction, None, cx).unwrap();
            assert!(remove_binding::<FirstAction>(cx));
            assert!(!remove_binding::<FirstAction>(cx));
        });

        assert!(actions_for("cmd-q", &app).is_empty());
    }

    fn actions_for(keystrokes: &str, app: &TestApp) -> Vec<&'static str> {
        app.read(|cx| {
            let keystrokes = keystrokes
                .split_whitespace()
                .map(Keystroke::parse)
                .collect::<Result<Vec<_>, _>>()
                .unwrap();
            let keymap = cx.key_bindings();
            let keymap = keymap.borrow();
            let (bindings, _) = keymap.bindings_for_input(&keystrokes, &[KeyContext::default()]);

            bindings
                .iter()
                .map(|binding| binding.action().name())
                .collect()
        })
    }
}
