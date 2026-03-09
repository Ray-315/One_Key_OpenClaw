use tauri::State;

use crate::error::AppError;
use crate::recipe::schema::{validate_recipe, Recipe, ValidationIssue};
use crate::state::AppState;

/// List all registered recipes.
#[tauri::command]
pub fn list_recipes(state: State<'_, AppState>) -> Result<Vec<Recipe>, AppError> {
    let registry = state
        .recipes
        .lock()
        .map_err(|e| AppError::Anyhow(anyhow::anyhow!("lock poisoned: {e}")))?;
    Ok(registry.list())
}

/// Load a recipe from a TOML file path and register it.
#[tauri::command]
pub fn load_recipe_file(
    state: State<'_, AppState>,
    path: String,
) -> Result<Recipe, AppError> {
    let mut registry = state
        .recipes
        .lock()
        .map_err(|e| AppError::Anyhow(anyhow::anyhow!("lock poisoned: {e}")))?;
    registry.load_file(&path)
}

/// Validate a recipe and return any issues.
#[tauri::command]
pub fn validate_recipe_cmd(
    _state: State<'_, AppState>,
    recipe: Recipe,
) -> Result<Vec<ValidationIssue>, AppError> {
    Ok(validate_recipe(&recipe))
}

/// Save (add or replace) a recipe in the registry.
#[tauri::command]
pub fn save_recipe(
    state: State<'_, AppState>,
    recipe: Recipe,
) -> Result<(), AppError> {
    let mut registry = state
        .recipes
        .lock()
        .map_err(|e| AppError::Anyhow(anyhow::anyhow!("lock poisoned: {e}")))?;
    registry.save(recipe);
    Ok(())
}

/// Delete a recipe from the registry by ID.
#[tauri::command]
pub fn delete_recipe(
    state: State<'_, AppState>,
    recipe_id: String,
) -> Result<(), AppError> {
    let mut registry = state
        .recipes
        .lock()
        .map_err(|e| AppError::Anyhow(anyhow::anyhow!("lock poisoned: {e}")))?;
    registry.delete(&recipe_id)
}

/// Fetch a recipe from a remote URL and register it.
#[tauri::command]
pub async fn fetch_recipe_url(
    state: State<'_, AppState>,
    url: String,
) -> Result<Recipe, AppError> {
    // Must drop the lock before the await, so fetch separately.
    let body = reqwest::get(&url)
        .await
        .map_err(|e| AppError::Anyhow(anyhow::anyhow!("HTTP fetch failed: {e}")))?
        .text()
        .await
        .map_err(|e| AppError::Anyhow(anyhow::anyhow!("failed to read body: {e}")))?;

    let recipe = crate::recipe::parser::parse_toml(&body)?;

    let mut registry = state
        .recipes
        .lock()
        .map_err(|e| AppError::Anyhow(anyhow::anyhow!("lock poisoned: {e}")))?;
    registry.save(recipe.clone());
    Ok(recipe)
}
