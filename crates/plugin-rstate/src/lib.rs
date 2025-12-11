use std::sync::Mutex;
use tauri::{
    Manager, Runtime,
    plugin::{Builder, TauriPlugin},
};

#[cfg(desktop)]
mod desktop;
#[cfg(mobile)]
mod mobile;

mod commands;
mod error;
mod models;
mod state_builder;

// Re-export core types
pub use crate::error::{Result, RstateError};
pub use crate::models::{Action, JsonValue, RstateManager, get_state, state_changed};
pub use crate::state_builder::{ActionHandler, BuiltStateManager, StateBuilder};

#[cfg(desktop)]
pub use desktop::{Rstate, STATE_UPDATE_EVENT};
#[cfg(mobile)]
pub use mobile::{Rstate, STATE_UPDATE_EVENT};

/// Extensions to [`tauri::App`], [`tauri::AppHandle`] and [`tauri::Window`] to access the rstate APIs.
pub trait RstateExt<R: Runtime> {
    fn rstate(&self) -> &Rstate<R>;
}

impl<R: Runtime, T: Manager<R>> crate::RstateExt<R> for T {
    fn rstate(&self) -> &Rstate<R> {
        self.state::<Rstate<R>>().inner()
    }
}

/// Type alias for the managed state.
///
/// Note: We don't need `Arc` here because Tauri's `app.manage()` wraps the state
/// in `Arc` internally. We only need `Mutex` for interior mutability.
pub type ManagedState = Mutex<Box<dyn RstateManager>>;

/// Initializes the plugin with a state manager.
///
/// # Example
///
/// ```rust,ignore
/// use tauri_plugin_rstate::{StateBuilder, init};
///
/// let state_manager = StateBuilder::new(AppState::default())
///     .on("INCREMENT", |state, _| { state.counter += 1; Ok(()) })
///     .build();
///
/// tauri::Builder::default()
///     .plugin(init(state_manager))
///     .run(tauri::generate_context!())
///     .unwrap();
/// ```
pub fn init<R: Runtime, S: RstateManager>(state_manager: S) -> TauriPlugin<R> {
    // Note: No Arc needed - Tauri handles Arc internally when we call app.manage()
    // We use Option + Mutex to allow taking ownership in the setup closure
    let state: ManagedState = Mutex::new(Box::new(state_manager));
    let state_cell = Mutex::new(Some(state));

    Builder::new("rstate")
        .invoke_handler(tauri::generate_handler![
            commands::get_initial_state,
            commands::get_state,
            commands::dispatch
        ])
        .setup(move |app, api| {
            #[cfg(mobile)]
            let rstate = mobile::init(app, api)?;
            #[cfg(desktop)]
            let rstate = desktop::init(app, api)?;

            // Take the state out of the Option (setup is only called once)
            if let Some(managed_state) = state_cell.lock().unwrap().take() {
                app.manage(managed_state);
            }
            app.manage(rstate);
            Ok(())
        })
        .build()
}

/// Initializes the plugin without a state manager.
///
/// You must register a state manager later using `app.rstate().register_state_manager()`.
///
/// # Example
///
/// ```rust,ignore
/// use tauri_plugin_rstate::{init_empty, RstateExt, StateBuilder};
///
/// tauri::Builder::default()
///     .plugin(init_empty())
///     .setup(|app| {
///         let manager = StateBuilder::new(AppState::default()).build();
///         app.rstate().register_state_manager(manager)?;
///         Ok(())
///     })
///     .run(tauri::generate_context!())
///     .unwrap();
/// ```
pub fn init_empty<R: Runtime>() -> TauriPlugin<R> {
    Builder::new("rstate")
        .invoke_handler(tauri::generate_handler![
            commands::get_initial_state,
            commands::get_state,
            commands::dispatch
        ])
        .setup(move |app, api| {
            #[cfg(mobile)]
            let rstate = mobile::init(app, api)?;
            #[cfg(desktop)]
            let rstate = desktop::init(app, api)?;

            // Only register the rstate interface, state manager will be registered later
            app.manage(rstate);
            Ok(())
        })
        .build()
}
