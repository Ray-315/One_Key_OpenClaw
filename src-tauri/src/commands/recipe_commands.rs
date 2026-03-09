use tauri::State;

use crate::error::AppError;
use crate::recipe::parser;
use crate::recipe::schema::{Recipe, ValidationIssue};
use crate::state::AppState;

/// List all registered recipes.
#[tauri::command]
pub fn list_recipes(state: State<'_, AppState>) -> Result<Vec<Recipe>, AppError> {
    state.recipe_registry.list()
}

/// Load and register a recipe from a TOML file path.
#[tauri::command]
pub fn load_recipe_file(
    state: State<'_, AppState>,
    path: String,
) -> Result<Recipe, AppError> {
    let recipe = parser::parse_file(&path)?;
    state.recipe_registry.upsert(recipe.clone())?;
    Ok(recipe)
}

/// Validate a recipe and return any issues found.
#[tauri::command]
pub fn validate_recipe(recipe: Recipe) -> Result<Vec<ValidationIssue>, AppError> {
    Ok(recipe.validate())
}

/// Save (upsert) a recipe into the registry.
#[tauri::command]
pub fn save_recipe(
    state: State<'_, AppState>,
    recipe: Recipe,
) -> Result<(), AppError> {
    state.recipe_registry.upsert(recipe)
}

/// Delete a recipe by id.
#[tauri::command]
pub fn delete_recipe(
    state: State<'_, AppState>,
    recipe_id: String,
) -> Result<(), AppError> {
    state.recipe_registry.remove(&recipe_id)
}

/// Get a single recipe by id.
#[tauri::command]
pub fn get_recipe(
    state: State<'_, AppState>,
    recipe_id: String,
) -> Result<Option<Recipe>, AppError> {
    state.recipe_registry.get(&recipe_id)
}
