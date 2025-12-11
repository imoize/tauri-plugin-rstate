mod state;

use tauri_plugin_rstate::{Action, RstateExt};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let state = state::create_state_manager();
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_rstate::init(state))
        .invoke_handler(tauri::generate_handler![
            increment_counter,
            decrement_counter,
            set_counter,
            add_todo_list,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[tauri::command]
async fn increment_counter<R: tauri::Runtime>(app: tauri::AppHandle<R>) -> Result<(), String> {
    // Method 1: dispatch_kind (no payload)
    app.rstate()
        .dispatch_kind("INCREMENT")
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
async fn decrement_counter<R: tauri::Runtime>(app: tauri::AppHandle<R>) -> Result<(), String> {
    app.rstate()
        .dispatch_kind("DECREMENT")
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
async fn set_counter<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    value: i32,
) -> Result<(), String> {
    // Method 2: dispatch_with (typed payload)
    app.rstate()
        .dispatch_with("SET_COUNTER", value)
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
async fn add_todo_list<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    text: String,
) -> Result<(), String> {
    // Method 3: dispatch (full Action)
    let action = Action::with_payload("ADD_TODO", text).map_err(|e| e.to_string())?;

    app.rstate().dispatch(action).map_err(|e| e.to_string())?;
    Ok(())
}
