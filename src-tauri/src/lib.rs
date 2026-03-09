mod commands;
mod env;
mod error;
mod log;
mod plugin;
mod recipe;
mod state;
mod task;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_shell::init())
        .manage(state::AppState::default())
        .invoke_handler(tauri::generate_handler![
            // env
            commands::env_commands::probe_all_envs,
            commands::env_commands::probe_env,
            // task
            commands::task_commands::start_task,
            commands::task_commands::pause_task,
            commands::task_commands::resume_task,
            commands::task_commands::cancel_task,
            commands::task_commands::get_task,
            commands::task_commands::list_tasks,
            // recipe
            commands::recipe_commands::list_recipes,
            commands::recipe_commands::load_recipe_file,
            commands::recipe_commands::validate_recipe_cmd,
            commands::recipe_commands::save_recipe,
            commands::recipe_commands::delete_recipe,
            commands::recipe_commands::fetch_recipe_url,
            // error / diagnostics
            commands::error_commands::diagnose_step_error,
            commands::error_commands::get_recipe_graph,
            // plugins
            commands::plugin_commands::list_plugins,
            commands::plugin_commands::load_plugin,
            commands::plugin_commands::unload_plugin,
            commands::plugin_commands::scan_plugins,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
