use tauri::State;

use crate::env::prober::{EnvItem, EnvProber};
use crate::error::AppError;
use crate::state::AppState;

/// Probe all supported environments, update the cache, and return results.
#[tauri::command]
pub fn probe_all_envs(state: State<'_, AppState>) -> Result<Vec<EnvItem>, AppError> {
    let items = EnvProber::probe_all();

    let mut cache = state
        .env_cache
        .lock()
        .map_err(|e| AppError::Anyhow(anyhow::anyhow!("Lock poisoned: {e}")))?;
    *cache = items.clone();

    Ok(items)
}

/// Probe a single environment by id, update its entry in the cache, and return it.
#[tauri::command]
pub fn probe_env(id: String, state: State<'_, AppState>) -> Result<EnvItem, AppError> {
    let item = EnvProber::probe(&id);

    let mut cache = state
        .env_cache
        .lock()
        .map_err(|e| AppError::Anyhow(anyhow::anyhow!("Lock poisoned: {e}")))?;

    // Update existing entry or push a new one.
    if let Some(existing) = cache.iter_mut().find(|e| e.id == id) {
        *existing = item.clone();
    } else {
        cache.push(item.clone());
    }

    Ok(item)
}
