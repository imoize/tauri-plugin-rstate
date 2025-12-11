use serde::{Deserialize, Serialize, de::DeserializeOwned};

pub use serde_json::Value as JsonValue;

/// An action to be dispatched to the state manager.
///
/// Actions are the primary way to modify state. Each action has a `kind` (type)
/// and an optional `payload` containing data for the action.
///
/// # Example
///
/// ```rust,ignore
/// use tauri_plugin_rstate::Action;
///
/// // Create a simple action
/// let action = Action::new("INCREMENT");
///
/// // Create an action with a payload
/// let action = Action::with_payload("ADD_TODO", "Buy groceries")?;
///
/// // In a handler, extract the payload
/// let text: String = action.require_payload()?;
/// ```
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Action {
    /// A string label for the action (e.g., "INCREMENT")
    pub kind: String,
    /// An optional payload for the action
    pub payload: Option<JsonValue>,
}

impl Action {
    /// Create a new action with a kind and no payload
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let action = Action::new("INCREMENT");
    /// ```
    pub fn new(kind: impl Into<String>) -> Self {
        Self {
            kind: kind.into(),
            payload: None,
        }
    }

    /// Create a new action with a kind and typed payload
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let action = Action::with_payload("SET_COUNT", 42)?;
    /// let action = Action::with_payload("ADD_TODO", TodoItem { text: "Buy milk".into() })?;
    /// ```
    pub fn with_payload<T: Serialize>(kind: impl Into<String>, payload: T) -> crate::Result<Self> {
        Ok(Self {
            kind: kind.into(),
            payload: Some(
                serde_json::to_value(payload)
                    .map_err(|e| crate::RstateError::serialization(e.to_string()))?,
            ),
        })
    }

    /// Create a new action with a kind and raw JSON payload
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use serde_json::json;
    /// let action = Action::with_json("SET_CONFIG", json!({"theme": "dark"}));
    /// ```
    pub fn with_json(kind: impl Into<String>, payload: JsonValue) -> Self {
        Self {
            kind: kind.into(),
            payload: Some(payload),
        }
    }

    /// Get the payload as a specific type, returning None if missing or invalid
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// if let Some(count) = action.payload_as::<i32>()? {
    ///     println!("Count: {}", count);
    /// }
    /// ```
    pub fn payload_as<T: DeserializeOwned>(&self) -> crate::Result<Option<T>> {
        match &self.payload {
            Some(value) => serde_json::from_value(value.clone())
                .map(Some)
                .map_err(|e| crate::RstateError::invalid_payload(e.to_string())),
            None => Ok(None),
        }
    }

    /// Get the payload as a specific type, returning an error if missing
    ///
    /// This is the most common way to extract a payload in action handlers.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// // In an action handler:
    /// let todo: TodoItem = action.require_payload()?;
    /// let count: i32 = action.require_payload()?;
    /// ```
    pub fn require_payload<T: DeserializeOwned>(&self) -> crate::Result<T> {
        match &self.payload {
            Some(value) => serde_json::from_value(value.clone())
                .map_err(|e| crate::RstateError::invalid_payload(e.to_string())),
            None => Err(crate::RstateError::missing_payload(&self.kind)),
        }
    }

    /// Check if the action has a payload
    pub fn has_payload(&self) -> bool {
        self.payload.is_some()
    }

    /// Check if the action matches a specific kind
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// if action.is("INCREMENT") {
    ///     // handle increment
    /// }
    /// ```
    pub fn is(&self, kind: &str) -> bool {
        self.kind == kind
    }
}

/// A trait that manages state for the app.
///
/// Implement this trait to define your state management logic.
/// For simpler use cases, consider using [`StateBuilder`] instead.
///
/// # Example
///
/// ```rust,ignore
/// use serde::{Deserialize, Serialize};
/// use std::sync::Mutex;
/// use tauri_plugin_rstate::{Action, JsonValue, RstateManager, Result, RstateError};
///
/// #[derive(Serialize, Deserialize, Default)]
/// struct AppState {
///     counter: i32,
/// }
///
/// struct MyStateManager {
///     state: Mutex<AppState>,
/// }
///
/// impl RstateManager for MyStateManager {
///     fn get_initial_state(&self) -> JsonValue {
///         self.state
///             .lock()
///             .ok()
///             .and_then(|s| serde_json::to_value(&*s).ok())
///             .unwrap_or(JsonValue::Null)
///     }
///
///     fn dispatch(&mut self, action: &Action) -> Result<JsonValue> {
///         let mut state = self.state.lock()
///             .map_err(|e| RstateError::LockPoisoned(e.to_string()))?;
///
///         match action.kind.as_str() {
///             "INCREMENT" => state.counter += 1,
///             "SET_COUNT" => state.counter = action.require_payload()?,
///             _ => {}
///         }
///
///         serde_json::to_value(&*state)
///             .map_err(|e| RstateError::serialization(e.to_string()))
///     }
/// }
/// ```
pub trait RstateManager: Send + Sync + 'static {
    /// Get the initial state of the app.
    fn get_initial_state(&self) -> JsonValue;

    /// Apply an action to the state and return the new state.
    fn dispatch(&mut self, action: &Action) -> crate::Result<JsonValue>;
}

/// Helper function to get a specific part of the state by key (supports dot notation).
///
/// # Example
///
/// ```rust,ignore
/// use serde_json::json;
/// use tauri_plugin_rstate::get_state;
///
/// let state = json!({
///     "user": {
///         "profile": {
///             "name": "John"
///         }
///     }
/// });
///
/// let name = get_state(&state, "user.profile.name");
/// assert_eq!(name, Some(json!("John")));
///
/// // Empty key returns the full state
/// let full = get_state(&state, "");
/// assert_eq!(full, Some(state.clone()));
/// ```
pub fn get_state(state: &JsonValue, key: &str) -> Option<JsonValue> {
    if key.is_empty() {
        return Some(state.clone());
    }

    // Convert the key to a JSON pointer path (e.g., "theme.is_dark" -> "/theme/is_dark")
    let pointer_path = format!("/{}", key.replace('.', "/"));

    // Use the built-in JSON pointer to get the value (more efficient)
    state.pointer(&pointer_path).cloned()
}

/// Helper function to check if a specific part of the state has changed.
///
/// This can be used for targeted updates when you only care about specific fields.
///
/// # Example
///
/// ```rust,ignore
/// if state_changed(&old_state, &new_state, "user.settings.theme") {
///     // Theme changed, update UI
/// }
/// ```
pub fn state_changed(old_state: &JsonValue, new_state: &JsonValue, key: &str) -> bool {
    let old_value = get_state(old_state, key);
    let new_value = get_state(new_state, key);

    match (old_value, new_value) {
        (Some(old), Some(new)) => old != new,
        (None, None) => false,
        _ => true, // One exists and the other doesn't, so it changed
    }
}
