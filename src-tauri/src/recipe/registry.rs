use std::collections::HashMap;

use crate::error::AppError;
use crate::recipe::parser;
use crate::recipe::schema::Recipe;

// Built-in recipes embedded at compile time.
const OPENCLAW_FULL_TOML: &str = include_str!("openclaw-full.toml");
const CLAUDE_CODE_TOML: &str = include_str!("claude-code.toml");

/// In-memory registry of all known recipes.
pub struct RecipeRegistry {
    recipes: HashMap<String, Recipe>,
}

impl RecipeRegistry {
    /// Create a registry pre-loaded with the built-in recipes.
    pub fn with_builtins() -> Self {
        let mut registry = Self {
            recipes: HashMap::new(),
        };
        registry.load_builtin(OPENCLAW_FULL_TOML, "openclaw-full.toml (built-in)");
        registry.load_builtin(CLAUDE_CODE_TOML, "claude-code.toml (built-in)");
        registry
    }

    fn load_builtin(&mut self, toml: &str, label: &str) {
        match parser::parse_toml(toml) {
            Ok(r) => {
                self.recipes.insert(r.id.clone(), r);
            }
            Err(e) => {
                eprintln!("[RecipeRegistry] Failed to load built-in {label}: {e}");
            }
        }
    }

    /// Return all registered recipes.
    pub fn list(&self) -> Vec<Recipe> {
        self.recipes.values().cloned().collect()
    }

    /// Look up a recipe by ID.
    pub fn get(&self, id: &str) -> Option<&Recipe> {
        self.recipes.get(id)
    }

    /// Add or replace a recipe.
    pub fn save(&mut self, recipe: Recipe) {
        self.recipes.insert(recipe.id.clone(), recipe);
    }

    /// Remove a recipe by ID. Returns an error if the recipe does not exist.
    pub fn delete(&mut self, id: &str) -> Result<(), AppError> {
        if self.recipes.remove(id).is_some() {
            Ok(())
        } else {
            Err(AppError::RecipeNotFound { recipe_id: id.into() })
        }
    }

    /// Load a recipe from a TOML file on disk and add it to the registry.
    pub fn load_file(&mut self, path: &str) -> Result<Recipe, AppError> {
        let recipe = parser::parse_file(path)?;
        self.recipes.insert(recipe.id.clone(), recipe.clone());
        Ok(recipe)
    }
}
