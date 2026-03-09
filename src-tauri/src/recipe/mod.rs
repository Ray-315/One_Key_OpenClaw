use std::collections::HashMap;
use std::sync::Mutex;

use crate::error::AppError;
use crate::recipe::schema::Recipe;

pub mod parser;
pub mod schema;

/// In-memory registry of loaded recipes.
pub struct RecipeRegistry {
    pub recipes: Mutex<HashMap<String, Recipe>>,
}

impl Default for RecipeRegistry {
    fn default() -> Self {
        Self {
            recipes: Mutex::new(HashMap::new()),
        }
    }
}

impl RecipeRegistry {
    /// Load the built-in recipes bundled into the binary.
    pub fn load_builtin(&self) -> Result<(), AppError> {
        let builtins: &[(&str, &str)] = &[
            (
                "openclaw-full",
                include_str!("../../assets/recipes/openclaw-full.toml"),
            ),
            (
                "claude-code",
                include_str!("../../assets/recipes/claude-code.toml"),
            ),
        ];

        let mut map = self.recipes.lock().map_err(|e| {
            AppError::Anyhow(anyhow::anyhow!("Failed to acquire recipe lock: {e}"))
        })?;

        for (id, source) in builtins {
            let recipe = parser::parse_toml(source)?;
            map.insert(id.to_string(), recipe);
        }
        Ok(())
    }

    /// Return all registered recipes.
    pub fn list(&self) -> Result<Vec<Recipe>, AppError> {
        let map = self.recipes.lock().map_err(|e| {
            AppError::Anyhow(anyhow::anyhow!("Failed to acquire recipe lock: {e}"))
        })?;
        Ok(map.values().cloned().collect())
    }

    /// Return a single recipe by id.
    pub fn get(&self, id: &str) -> Result<Option<Recipe>, AppError> {
        let map = self.recipes.lock().map_err(|e| {
            AppError::Anyhow(anyhow::anyhow!("Failed to acquire recipe lock: {e}"))
        })?;
        Ok(map.get(id).cloned())
    }

    /// Insert or replace a recipe.
    pub fn upsert(&self, recipe: Recipe) -> Result<(), AppError> {
        let mut map = self.recipes.lock().map_err(|e| {
            AppError::Anyhow(anyhow::anyhow!("Failed to acquire recipe lock: {e}"))
        })?;
        map.insert(recipe.id.clone(), recipe);
        Ok(())
    }

    /// Remove a recipe by id.
    pub fn remove(&self, id: &str) -> Result<(), AppError> {
        let mut map = self.recipes.lock().map_err(|e| {
            AppError::Anyhow(anyhow::anyhow!("Failed to acquire recipe lock: {e}"))
        })?;
        map.remove(id);
        Ok(())
    }
}
