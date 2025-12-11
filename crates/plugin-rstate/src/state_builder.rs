//! A builder pattern for creating state managers with less boilerplate.
//!
//! This module provides a [`StateBuilder`] that allows you to define state
//! and action handlers in a more declarative way, reducing the amount of
//! manual code needed compared to implementing [`RstateManager`] directly.
//!
//! # Example
//!
//! ```rust,ignore
//! use serde::{Deserialize, Serialize};
//! use tauri_plugin_rstate::{StateBuilder, RstateError};
//!
//! #[derive(Serialize, Deserialize, Default, Clone)]
//! struct AppState {
//!     counter: i32,
//!     todos: Vec<String>,
//! }
//!
//! let state_manager = StateBuilder::new(AppState::default())
//!     .on("INCREMENT", |state, _action| {
//!         state.counter += 1;
//!         Ok(())
//!     })
//!     .on("DECREMENT", |state, _action| {
//!         state.counter -= 1;
//!         Ok(())
//!     })
//!     .on("ADD_TODO", |state, action| {
//!         let text: String = action.require_payload()?;
//!         state.todos.push(text);
//!         Ok(())
//!     })
//!     .on_default(|_state, action| {
//!         Err(RstateError::action_not_found(&action.kind))
//!     })
//!     .build();
//!
//! // Use with the plugin
//! tauri::Builder::default()
//!     .plugin(tauri_plugin_rstate::init(state_manager))
//!     .run(tauri::generate_context!())
//!     .expect("error while running tauri application");
//! ```

use serde::{Serialize, de::DeserializeOwned};
use std::collections::HashMap;
use std::sync::Mutex;

use crate::Result;
use crate::models::{Action, JsonValue, RstateManager};

/// A handler function type for processing actions.
///
/// The handler receives a mutable reference to the state and the action,
/// and should return `Ok(())` on success or an error if the action failed.
pub type ActionHandler<T> = Box<dyn Fn(&mut T, &Action) -> Result<()> + Send + Sync>;

/// A builder for creating state managers with a fluent API.
///
/// `StateBuilder` provides a declarative way to define your state and action handlers
/// without having to manually implement the [`RstateManager`] trait.
///
/// # Type Parameters
///
/// * `T` - The state type, which must be serializable and deserializable.
///
/// # Example
///
/// ```rust,ignore
/// use serde::{Deserialize, Serialize};
/// use tauri_plugin_rstate::StateBuilder;
///
/// #[derive(Serialize, Deserialize, Default)]
/// struct Counter {
///     value: i32,
/// }
///
/// let manager = StateBuilder::new(Counter::default())
///     .on("INCREMENT", |state, _| {
///         state.value += 1;
///         Ok(())
///     })
///     .on("SET", |state, action| {
///         state.value = action.require_payload()?;
///         Ok(())
///     })
///     .build();
/// ```
pub struct StateBuilder<T>
where
    T: Serialize + DeserializeOwned + Send + Sync + 'static,
{
    initial_state: T,
    handlers: HashMap<String, ActionHandler<T>>,
    default_handler: Option<ActionHandler<T>>,
}

impl<T> StateBuilder<T>
where
    T: Serialize + DeserializeOwned + Send + Sync + 'static,
{
    /// Create a new `StateBuilder` with the given initial state.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let builder = StateBuilder::new(AppState::default());
    /// ```
    pub fn new(initial_state: T) -> Self {
        Self {
            initial_state,
            handlers: HashMap::new(),
            default_handler: None,
        }
    }

    /// Register an action handler for a specific action kind.
    ///
    /// The handler will be called when an action with the matching `kind` is dispatched.
    /// The handler receives a mutable reference to the state and the action.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// builder
    ///     .on("INCREMENT", |state, _action| {
    ///         state.counter += 1;
    ///         Ok(())
    ///     })
    ///     .on("SET_VALUE", |state, action| {
    ///         state.value = action.require_payload()?;
    ///         Ok(())
    ///     })
    /// ```
    #[must_use]
    pub fn on<F>(mut self, action_kind: impl Into<String>, handler: F) -> Self
    where
        F: Fn(&mut T, &Action) -> Result<()> + Send + Sync + 'static,
    {
        self.handlers.insert(action_kind.into(), Box::new(handler));
        self
    }

    /// Register a default handler for unknown actions.
    ///
    /// This handler is called when no specific handler is found for an action.
    /// If not set, unknown actions will be silently ignored (state unchanged).
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// // Log unknown actions but don't fail
    /// builder.on_default(|_state, action| {
    ///     log::warn!("Unknown action: {}", action.kind);
    ///     Ok(())
    /// })
    ///
    /// // Or fail on unknown actions
    /// builder.on_default(|_state, action| {
    ///     Err(RstateError::action_not_found(&action.kind))
    /// })
    /// ```
    #[must_use]
    pub fn on_default<F>(mut self, handler: F) -> Self
    where
        F: Fn(&mut T, &Action) -> Result<()> + Send + Sync + 'static,
    {
        self.default_handler = Some(Box::new(handler));
        self
    }

    /// Build the state manager.
    ///
    /// Returns a [`BuiltStateManager`] that implements [`RstateManager`]
    /// and can be passed to [`crate::init`].
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let state_manager = StateBuilder::new(AppState::default())
    ///     .on("INCREMENT", |state, _| { state.counter += 1; Ok(()) })
    ///     .build();
    ///
    /// tauri::Builder::default()
    ///     .plugin(tauri_plugin_rstate::init(state_manager))
    ///     .run(tauri::generate_context!())
    ///     .unwrap();
    /// ```
    pub fn build(self) -> BuiltStateManager<T> {
        BuiltStateManager {
            state: Mutex::new(self.initial_state),
            handlers: self.handlers,
            default_handler: self.default_handler,
        }
    }
}

impl<T> Default for StateBuilder<T>
where
    T: Serialize + DeserializeOwned + Send + Sync + Default + 'static,
{
    fn default() -> Self {
        Self::new(T::default())
    }
}

/// A state manager built from [`StateBuilder`].
///
/// This struct implements [`RstateManager`] and handles:
/// - Thread-safe state access via internal [`Mutex`]
/// - Action routing to registered handlers
/// - Automatic serialization of state to JSON
///
/// You typically don't create this directly; use [`StateBuilder::build`] instead.
pub struct BuiltStateManager<T>
where
    T: Serialize + DeserializeOwned + Send + Sync + 'static,
{
    state: Mutex<T>,
    handlers: HashMap<String, ActionHandler<T>>,
    default_handler: Option<ActionHandler<T>>,
}

impl<T> BuiltStateManager<T>
where
    T: Serialize + DeserializeOwned + Send + Sync + 'static,
{
    /// Execute a function with a reference to the current state.
    ///
    /// This locks the internal mutex for the duration of the function call.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let count = manager.with_state(|state| state.counter)?;
    /// ```
    pub fn with_state<R, F: FnOnce(&T) -> R>(&self, f: F) -> Result<R> {
        let state = self
            .state
            .lock()
            .map_err(|e| crate::RstateError::LockPoisoned(e.to_string()))?;
        Ok(f(&state))
    }

    /// Execute a function with a mutable reference to the current state.
    ///
    /// This locks the internal mutex for the duration of the function call.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// manager.with_state_mut(|state| {
    ///     state.counter += 1;
    /// })?;
    /// ```
    pub fn with_state_mut<R, F: FnOnce(&mut T) -> R>(&self, f: F) -> Result<R> {
        let mut state = self
            .state
            .lock()
            .map_err(|e| crate::RstateError::LockPoisoned(e.to_string()))?;
        Ok(f(&mut state))
    }

    /// Get a clone of the current state.
    ///
    /// This is useful when you need to read the state outside of a handler.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let current_state: AppState = manager.get_state_clone()?;
    /// ```
    pub fn get_state_clone(&self) -> Result<T>
    where
        T: Clone,
    {
        self.with_state(|s| s.clone())
    }
}

impl<T> RstateManager for BuiltStateManager<T>
where
    T: Serialize + DeserializeOwned + Send + Sync + 'static,
{
    fn get_initial_state(&self) -> JsonValue {
        // Safe: if lock is poisoned, return Null rather than panic
        self.state
            .lock()
            .ok()
            .and_then(|state| serde_json::to_value(&*state).ok())
            .unwrap_or(JsonValue::Null)
    }

    fn dispatch(&mut self, action: &Action) -> Result<JsonValue> {
        let mut state = self
            .state
            .lock()
            .map_err(|e| crate::RstateError::LockPoisoned(e.to_string()))?;

        // Find and execute the handler
        if let Some(handler) = self.handlers.get(&action.kind) {
            handler(&mut state, action)?;
        } else if let Some(ref default_handler) = self.default_handler {
            default_handler(&mut state, action)?;
        }
        // If no handler found and no default, silently ignore (state unchanged)

        // Return updated state
        serde_json::to_value(&*state).map_err(|e| crate::RstateError::serialization(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, Default, Clone, Debug, PartialEq)]
    struct TestState {
        counter: i32,
        message: String,
    }

    #[test]
    fn test_state_builder_basic() {
        let mut manager = StateBuilder::new(TestState::default())
            .on("INCREMENT", |state, _| {
                state.counter += 1;
                Ok(())
            })
            .on("SET_MESSAGE", |state, action| {
                state.message = action.require_payload()?;
                Ok(())
            })
            .build();

        // Test initial state
        let initial = manager.get_initial_state();
        assert_eq!(initial["counter"], 0);
        assert_eq!(initial["message"], "");

        // Test INCREMENT
        let action = Action::new("INCREMENT");
        let result = manager.dispatch(&action).unwrap();
        assert_eq!(result["counter"], 1);

        // Test SET_MESSAGE
        let action = Action::with_payload("SET_MESSAGE", "Hello").unwrap();
        let result = manager.dispatch(&action).unwrap();
        assert_eq!(result["message"], "Hello");
    }

    #[test]
    fn test_state_builder_default_handler() {
        let mut manager = StateBuilder::new(TestState::default())
            .on_default(|_state, action| Err(crate::RstateError::action_not_found(&action.kind)))
            .build();

        let action = Action::new("UNKNOWN");
        let result = manager.dispatch(&action);
        assert!(result.is_err());
    }

    #[test]
    fn test_state_builder_unknown_action_ignored() {
        let mut manager = StateBuilder::new(TestState {
            counter: 5,
            message: "test".into(),
        })
        .on("INCREMENT", |state, _| {
            state.counter += 1;
            Ok(())
        })
        .build();

        // Unknown action should be silently ignored
        let action = Action::new("UNKNOWN");
        let result = manager.dispatch(&action).unwrap();
        assert_eq!(result["counter"], 5); // Unchanged
    }

    #[test]
    fn test_action_helpers() {
        // Test Action::new
        let action = Action::new("TEST");
        assert_eq!(action.kind, "TEST");
        assert!(!action.has_payload());

        // Test Action::with_payload
        let action = Action::with_payload("SET", 42i32).unwrap();
        assert_eq!(action.kind, "SET");
        assert!(action.has_payload());
        assert_eq!(action.require_payload::<i32>().unwrap(), 42);

        // Test Action::is
        assert!(action.is("SET"));
        assert!(!action.is("GET"));

        // Test payload_as
        let opt: Option<i32> = action.payload_as().unwrap();
        assert_eq!(opt, Some(42));
    }
}
