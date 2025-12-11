use serde::{Deserialize, Serialize};
use tauri_plugin_rstate::{RstateError, RstateManager, StateBuilder};

// 1. Define your state struct
#[derive(Serialize, Deserialize, Default)]
pub struct AppState {
    pub counter: i32,
    pub message: String,
    pub todos: Vec<TodoItem>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct TodoItem {
    pub id: u32,
    pub text: String,
    pub completed: bool,
}

pub fn create_state_manager() -> impl RstateManager {
    StateBuilder::new(AppState::default())
        // Simple action without payload
        .on("INCREMENT", |state, _action| {
            state.counter += 1;
            Ok(())

        })
        .on("DECREMENT", |state, _action| {
            state.counter -= 1;
            Ok(())
        })

        // Action with typed payload
        .on("SET_COUNTER", |state, action| {
            // Automatically deserializes and validates the payload
            state.counter = action.require_payload()?;
            Ok(())
        })

        // Action with complex payload
        .on("ADD_TODO", |state, action| {
            let text: String = action.require_payload()?;
            let id = state.todos.len() as u32 + 1;
            state.todos.push(TodoItem {
                id,
                text,
                completed: false,
            });
            Ok(())
        })

        // Handle unknown actions (optional)
        .on_default(|_state, action| {
            // You can either:
            // 1. Silently ignore: Ok(())
            // 2. Log warning: log::warn!("Unknown: {}", action.kind); Ok(())
            // 3. Return error:
            Err(RstateError::action_not_found(&action.kind))
        })

        .build()
}
