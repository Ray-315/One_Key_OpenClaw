use tauri::State;

use crate::error::AppError;
use crate::plugin::api::PluginInfo;
use crate::state::AppState;

/// List all loaded plugins.
#[tauri::command]
pub fn list_plugins(state: State<'_, AppState>) -> Result<Vec<PluginInfo>, AppError> {
    let mgr = state
        .plugins
        .lock()
        .map_err(|e| AppError::Anyhow(anyhow::anyhow!("lock poisoned: {e}")))?;
    Ok(mgr.list_plugins())
}

/// Load a plugin from a directory that contains a `plugin.toml`.
#[tauri::command]
pub fn load_plugin(state: State<'_, AppState>, path: String) -> Result<PluginInfo, AppError> {
    let mut mgr = state
        .plugins
        .lock()
        .map_err(|e| AppError::Anyhow(anyhow::anyhow!("lock poisoned: {e}")))?;
    let manifest_path = std::path::Path::new(&path).join("plugin.toml");
    mgr.load_plugin(&manifest_path)
}

/// Unload a plugin by its ID.
#[tauri::command]
pub fn unload_plugin(state: State<'_, AppState>, plugin_id: String) -> Result<(), AppError> {
    let mut mgr = state
        .plugins
        .lock()
        .map_err(|e| AppError::Anyhow(anyhow::anyhow!("lock poisoned: {e}")))?;
    mgr.unload_plugin(&plugin_id)
}

/// Re-scan the plugins directory and load any new plugins.
#[tauri::command]
pub fn scan_plugins(state: State<'_, AppState>) -> Result<usize, AppError> {
    let mut mgr = state
        .plugins
        .lock()
        .map_err(|e| AppError::Anyhow(anyhow::anyhow!("lock poisoned: {e}")))?;
    mgr.scan_plugins()
}
