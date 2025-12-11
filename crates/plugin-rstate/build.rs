const COMMANDS: &[&str] = &["get_initial_state", "get_state", "dispatch"];

fn main() {
    tauri_plugin::Builder::new(COMMANDS)
        .android_path("android")
        .ios_path("ios")
        .build();
}
