use serde::de::DeserializeOwned;
use std::sync::Mutex;
use tauri::{
    AppHandle, Manager, Runtime,
    plugin::{PluginApi, PluginHandle},
};

use crate::ManagedState;
use crate::models::*;

#[cfg(target_os = "ios")]
tauri::ios_plugin_binding!(init_plugin_rstate);

/// Initializes the mobile plugin.
pub fn init<R: Runtime, C: DeserializeOwned>(
    app: &AppHandle<R>,
    api: PluginApi<R, C>,
) -> crate::Result<Rstate<R>> {
    #[cfg(target_os = "android")]
    let handle = api.register_android_plugin("", "ExamplePlugin")?;
    #[cfg(target_os = "ios")]
    let handle = api.register_ios_plugin(init_plugin_rstate)?;
    Ok(Rstate {
        handle,
        app: app.clone(),
    })
}

/// Event name used for state updates.
pub const STATE_UPDATE_EVENT: &str = "rstate://state-update";

/// Access to the rstate APIs on mobile.
pub struct Rstate<R: Runtime> {
    #[allow(dead_code)]
    handle: PluginHandle<R>,
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
    /// Note: Tauri wraps managed state in Arc internally
    #[inline]
    pub fn is_registered(&self) -> bool {
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
    pub fn get_initial_state(&self) -> crate::Result<JsonValue> {
        let state_manager = self.state_manager()?;
        let state_guard = state_manager
            .lock()
            .map_err(|e| crate::RstateError::LockPoisoned(e.to_string()))?;
        Ok(state_guard.get_initial_state())
    }

    /// Get a specific part of the state by key.
    pub fn get_state(&self, key: &str) -> crate::Result<Option<JsonValue>> {
        let state_manager = self.state_manager()?;
        let state_guard = state_manager
            .lock()
            .map_err(|e| crate::RstateError::LockPoisoned(e.to_string()))?;
        let full_state = state_guard.get_initial_state();
        Ok(crate::models::get_state(&full_state, key))
    }

    /// Dispatch an action to the state manager.
    pub fn dispatch(&self, action: Action) -> crate::Result<JsonValue> {
        let state_manager = self.state_manager()?;
        let mut state_guard = state_manager
            .lock()
            .map_err(|e| crate::RstateError::LockPoisoned(e.to_string()))?;
        state_guard.dispatch(&action)
    }

    /// Dispatch an action with just a kind (no payload).
    #[inline]
    pub fn dispatch_kind(&self, kind: impl Into<String>) -> crate::Result<JsonValue> {
        self.dispatch(Action::new(kind))
    }

    /// Dispatch an action with a typed payload.
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
    /// Note: Tauri handles Arc internally when we call `app.manage()`,
    /// so we only wrap in Mutex for interior mutability.
    pub fn register_state_manager<S: RstateManager>(&self, state_manager: S) -> crate::Result<()> {
        let state: ManagedState = Mutex::new(Box::new(state_manager));
        self.app.manage(state);
        Ok(())
    }
}
