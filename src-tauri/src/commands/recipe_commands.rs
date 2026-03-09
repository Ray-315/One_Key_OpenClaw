use std::time::Duration;
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
pub fn load_recipe_file(state: State<'_, AppState>, path: String) -> Result<Recipe, AppError> {
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
pub fn save_recipe(state: State<'_, AppState>, recipe: Recipe) -> Result<(), AppError> {
    let mut registry = state
        .recipes
        .lock()
        .map_err(|e| AppError::Anyhow(anyhow::anyhow!("lock poisoned: {e}")))?;
    registry.save(recipe);
    Ok(())
}

/// Delete a recipe from the registry by ID.
#[tauri::command]
pub fn delete_recipe(state: State<'_, AppState>, recipe_id: String) -> Result<(), AppError> {
    let mut registry = state
        .recipes
        .lock()
        .map_err(|e| AppError::Anyhow(anyhow::anyhow!("lock poisoned: {e}")))?;
    registry.delete(&recipe_id)
}

/// Maximum response body size for remote recipe fetch (1 MB).
const MAX_RECIPE_BODY_SIZE: usize = 1_024 * 1_024;

/// Fetch a recipe from a remote URL and register it.
#[tauri::command]
pub async fn fetch_recipe_url(state: State<'_, AppState>, url: String) -> Result<Recipe, AppError> {
    // Validate URL scheme – only allow HTTPS (and HTTP for localhost).
    let parsed = reqwest::Url::parse(&url)
        .map_err(|e| AppError::Anyhow(anyhow::anyhow!("invalid URL: {e}")))?;
    match parsed.scheme() {
        "https" => {}
        "http"
            if parsed
                .host_str()
                .map_or(false, |h| h == "localhost" || h == "127.0.0.1") => {}
        other => {
            return Err(AppError::Anyhow(anyhow::anyhow!(
                "unsupported URL scheme '{other}': only HTTPS is allowed"
            )));
        }
    }

    // Fetch with a size limit to prevent resource exhaustion.
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(|e| AppError::Anyhow(anyhow::anyhow!("failed to build HTTP client: {e}")))?;

    let mut resp = client
        .get(parsed)
        .send()
        .await
        .and_then(reqwest::Response::error_for_status)
        .map_err(|e| AppError::Anyhow(anyhow::anyhow!("HTTP fetch failed: {e}")))?;

    if resp
        .content_length()
        .is_some_and(|content_length| content_length as usize > MAX_RECIPE_BODY_SIZE)
    {
        return Err(AppError::Anyhow(anyhow::anyhow!(
            "response too large (max {MAX_RECIPE_BODY_SIZE} bytes)"
        )));
    }

    let mut body = Vec::new();
    while let Some(chunk) = resp
        .chunk()
        .await
        .map_err(|e| AppError::Anyhow(anyhow::anyhow!("failed to read body: {e}")))?
    {
        if body.len() + chunk.len() > MAX_RECIPE_BODY_SIZE {
            return Err(AppError::Anyhow(anyhow::anyhow!(
                "response body too large (max {MAX_RECIPE_BODY_SIZE} bytes)"
            )));
        }
        body.extend_from_slice(&chunk);
    }

    let body = String::from_utf8(body)
        .map_err(|e| AppError::Anyhow(anyhow::anyhow!("response body is not valid UTF-8: {e}")))?;

    let recipe = crate::recipe::parser::parse_toml(&body)?;

    let mut registry = state
        .recipes
        .lock()
        .map_err(|e| AppError::Anyhow(anyhow::anyhow!("lock poisoned: {e}")))?;
    registry.save(recipe.clone());
    Ok(recipe)
}
