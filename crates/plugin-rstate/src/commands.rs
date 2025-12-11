use tauri::{AppHandle, Runtime, command};

use crate::Result;
use crate::RstateExt;
use crate::models::{Action, JsonValue};

/// Get the initial/full state.
#[command]
pub(crate) fn get_initial_state<R: Runtime>(app: AppHandle<R>) -> Result<JsonValue> {
    app.rstate().get_initial_state()
}

/// Get a specific part of the state by key.
#[command]
pub(crate) fn get_state<R: Runtime>(app: AppHandle<R>, key: &str) -> Result<Option<JsonValue>> {
    app.rstate().get_state(key)
}

/// Dispatch an action to modify the state.
#[command]
pub(crate) fn dispatch<R: Runtime>(app: AppHandle<R>, action: Action) -> Result<JsonValue> {
    app.rstate().dispatch(action)
}
