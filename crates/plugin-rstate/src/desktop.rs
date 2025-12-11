use serde::de::DeserializeOwned;
use std::sync::Mutex;
use tauri::{AppHandle, Emitter, Manager, Runtime, plugin::PluginApi};

use crate::ManagedState;
use crate::models::{Action, JsonValue, RstateManager};

/// Event name used for state updates.
pub const STATE_UPDATE_EVENT: &str = "rstate://state-update";

// Compare two JSON values for equality (deep comparison).
// Prevents unnecessary state update events when values haven't changed.
fn states_are_equal(current: &JsonValue, updated: &JsonValue) -> bool {
    match (current, updated) {
        // Both are null
        (JsonValue::Null, JsonValue::Null) => true,

        // Both are booleans
        (JsonValue::Bool(a), JsonValue::Bool(b)) => a == b,

        // Both are numbers - compare as integers first, then as floats
        (JsonValue::Number(a), JsonValue::Number(b)) => {
            // Try to compare as integers first, then as floats
            if let (Some(a_int), Some(b_int)) = (a.as_i64(), b.as_i64()) {
                a_int == b_int
            } else if let (Some(a_float), Some(b_float)) = (a.as_f64(), b.as_f64()) {
                (a_float - b_float).abs() < f64::EPSILON
            } else {
                false
            }
        }

        // Both are strings
        (JsonValue::String(a), JsonValue::String(b)) => a == b,

        // Both are arrays - compare length and each element
        (JsonValue::Array(a), JsonValue::Array(b)) => {
            a.len() == b.len()
                && a.iter()
                    .zip(b.iter())
                    .all(|(a_item, b_item)| states_are_equal(a_item, b_item))
        }

        // Both are objects - compare all key-value pairs
        (JsonValue::Object(a), JsonValue::Object(b)) => {
            a.len() == b.len()
                && a.iter().all(|(key, a_value)| {
                    b.get(key)
                        .is_some_and(|b_value| states_are_equal(a_value, b_value))
                })
        }

        // Different types or one is null and the other isn't
        _ => false,
    }
}

pub fn init<R: Runtime, C: DeserializeOwned>(
    app: &AppHandle<R>,
    _api: PluginApi<R, C>,
) -> crate::Result<Rstate<R>> {
    Ok(Rstate { app: app.clone() })
}

/// Access to the rstate APIs.
///
/// This struct provides synchronous methods for state management.
/// Access it via the [`RstateExt`](crate::RstateExt) trait.
///
/// # Example
///
/// ```rust,ignore
/// use tauri_plugin_rstate::RstateExt;
///
/// #[tauri::command]
/// fn increment(app: tauri::AppHandle) -> Result<(), String> {
///     app.rstate()
///         .dispatch_kind("INCREMENT")
///         .map_err(|e| e.to_string())?;
///     Ok(())
/// }
/// ```
pub struct Rstate<R: Runtime> {
    app: AppHandle<R>,
}

impl<R: Runtime> Rstate<R> {
    /// Get the event name used for state updates.
    #[inline]
    pub fn get_event_name(&self) -> &'static str {
        STATE_UPDATE_EVENT
    }

    /// Check if a state manager is registered.
    ///
    /// Returns `true` if a state manager has been registered, `false` otherwise.
    #[inline]
    pub fn is_registered(&self) -> bool {
        // Note: Tauri wraps managed state in Arc internally
        self.app.try_state::<ManagedState>().is_some()
    }

    // Helper to get the state manager
    // Note: Tauri handles Arc internally, we only need Mutex for interior mutability
    #[inline]
    fn state_manager(&self) -> crate::Result<tauri::State<'_, ManagedState>> {
        self.app
            .try_state::<ManagedState>()
            .ok_or(crate::RstateError::NotRegistered)
    }

    /// Get the initial state from the state manager.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let state = app.rstate().get_initial_state()?;
    /// ```
    pub fn get_initial_state(&self) -> crate::Result<JsonValue> {
        let state_manager = self.state_manager()?;
        let state_guard = state_manager
            .lock()
            .map_err(|e| crate::RstateError::LockPoisoned(e.to_string()))?;
        Ok(state_guard.get_initial_state())
    }

    /// Get a specific part of the state by key (supports dot notation).
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// // Get nested state
    /// let user_name = app.rstate().get_state("user.profile.name")?;
    /// ```
    pub fn get_state(&self, key: &str) -> crate::Result<Option<JsonValue>> {
        let state_manager = self.state_manager()?;
        let state_guard = state_manager
            .lock()
            .map_err(|e| crate::RstateError::LockPoisoned(e.to_string()))?;
        let full_state = state_guard.get_initial_state();
        Ok(crate::models::get_state(&full_state, key))
    }

    /// Dispatch an action to the state manager.
    ///
    /// Emits a state update event only if the state actually changed.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use tauri_plugin_rstate::Action;
    ///
    /// let action = Action::with_payload("SET_COUNT", 42)?;
    /// let new_state = app.rstate().dispatch(action)?;
    /// ```
    pub fn dispatch(&self, action: Action) -> crate::Result<JsonValue> {
        let state_manager = self.state_manager()?;

        // Hold the lock for the minimum time necessary
        let (current_state, updated_state) = {
            let mut state_guard = state_manager
                .lock()
                .map_err(|e| crate::RstateError::LockPoisoned(e.to_string()))?;

            // Get current state for comparison
            let current = state_guard.get_initial_state();

            // Dispatch action
            let updated = state_guard.dispatch(&action)?;

            (current, updated)
        };
        // Lock is released here

        // Only emit state update if the state actually changed
        if !states_are_equal(&current_state, &updated_state) {
            self.app
                .emit(STATE_UPDATE_EVENT, &updated_state)
                .map_err(|err| crate::RstateError::Emit(err.to_string()))?;
        }

        Ok(updated_state)
    }

    /// Dispatch an action with just a kind (no payload).
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// app.rstate().dispatch_kind("INCREMENT")?;
    /// ```
    #[inline]
    pub fn dispatch_kind(&self, kind: impl Into<String>) -> crate::Result<JsonValue> {
        self.dispatch(Action::new(kind))
    }

    /// Dispatch an action with a typed payload.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// app.rstate().dispatch_with("SET_COUNT", 42)?;
    /// app.rstate().dispatch_with("ADD_TODO", &todo_item)?;
    /// ```
    #[inline]
    pub fn dispatch_with<T: serde::Serialize>(
        &self,
        kind: impl Into<String>,
        payload: T,
    ) -> crate::Result<JsonValue> {
        self.dispatch(Action::with_payload(kind, payload)?)
    }

    /// Register a state manager.
    ///
    /// Use this with [`init_empty`](crate::init_empty) for lazy initialization.
    ///
    /// Note: Tauri handles Arc internally when we call `app.manage()`,
    /// so we only wrap in Mutex for interior mutability.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let manager = StateBuilder::new(AppState::default()).build();
    /// app.rstate().register_state_manager(manager)?;
    /// ```
    pub fn register_state_manager<S: RstateManager>(&self, state_manager: S) -> crate::Result<()> {
        let state: ManagedState = Mutex::new(Box::new(state_manager));
        self.app.manage(state);
        Ok(())
    }
}
