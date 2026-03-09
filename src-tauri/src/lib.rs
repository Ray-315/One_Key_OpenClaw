mod commands;
mod env;
mod error;
mod log;
mod state;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_shell::init())
        .manage(state::AppState::new())
        .invoke_handler(tauri::generate_handler![
            commands::env_commands::probe_all_envs,
            commands::env_commands::probe_env,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
