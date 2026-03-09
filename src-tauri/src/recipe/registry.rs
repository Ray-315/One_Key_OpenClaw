use std::collections::HashMap;
use std::path::{Path, PathBuf};

use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use tokio::sync::mpsc;

use crate::error::AppError;
use crate::recipe::parser;
use crate::recipe::schema::Recipe;

// Built-in recipes embedded at compile time.
const OPENCLAW_FULL_TOML: &str = include_str!("openclaw-full.toml");
const CLAUDE_CODE_TOML: &str = include_str!("claude-code.toml");

/// In-memory registry of all known recipes.
pub struct RecipeRegistry {
    recipes: HashMap<String, Recipe>,
    /// Local recipes directory for hot-reload watching.
    local_dir: Option<PathBuf>,
    /// File watcher handle (kept alive to maintain the watch).
    #[allow(dead_code)]
    watcher: Option<RecommendedWatcher>,
}

impl RecipeRegistry {
    /// Create a registry pre-loaded with the built-in recipes.
    pub fn with_builtins() -> Self {
        let mut registry = Self {
            recipes: HashMap::new(),
            local_dir: None,
            watcher: None,
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
            Err(AppError::RecipeNotFound {
                recipe_id: id.into(),
            })
        }
    }

    /// Load a recipe from a TOML file on disk and add it to the registry.
    pub fn load_file(&mut self, path: &str) -> Result<Recipe, AppError> {
        let recipe = parser::parse_file(path)?;
        self.recipes.insert(recipe.id.clone(), recipe.clone());
        Ok(recipe)
    }

    // -----------------------------------------------------------------------
    // Phase-4 additions
    // -----------------------------------------------------------------------

    /// Fetch a recipe from a remote URL (must return TOML).
    pub async fn fetch_from_url(&mut self, url: &str) -> Result<Recipe, AppError> {
        let body = reqwest::get(url)
            .await
            .map_err(|e| AppError::Anyhow(anyhow::anyhow!("HTTP fetch failed: {e}")))?
            .text()
            .await
            .map_err(|e| AppError::Anyhow(anyhow::anyhow!("failed to read body: {e}")))?;

        let recipe = parser::parse_toml(&body)?;
        self.recipes.insert(recipe.id.clone(), recipe.clone());
        Ok(recipe)
    }

    /// Scan `local_dir` for `.toml` recipe files and load them all.
    pub fn load_local_recipes(&mut self, dir: &Path) -> Result<usize, AppError> {
        if !dir.exists() {
            return Ok(0);
        }
        self.local_dir = Some(dir.to_path_buf());

        let entries = std::fs::read_dir(dir).map_err(AppError::IoError)?;
        let mut count = 0usize;
        for entry in entries {
            let entry = entry.map_err(AppError::IoError)?;
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("toml") {
                match self.load_file(&path.to_string_lossy()) {
                    Ok(_) => count += 1,
                    Err(e) => {
                        eprintln!(
                            "[RecipeRegistry] Failed to load recipe {}: {e}",
                            path.display()
                        );
                    }
                }
            }
        }
        Ok(count)
    }

    /// Watch the local recipes directory for changes and send the id of any
    /// changed recipe over the channel so the caller can re-emit events.
    pub fn watch_local_dir(
        &mut self,
        dir: PathBuf,
        change_tx: mpsc::UnboundedSender<String>,
    ) -> Result<(), AppError> {
        self.local_dir = Some(dir.clone());

        let dir_clone = dir.clone();
        let mut watcher =
            notify::recommended_watcher(move |res: Result<notify::Event, notify::Error>| {
                if let Ok(event) = res {
                    for path in &event.paths {
                        if path.extension().and_then(|e| e.to_str()) == Some("toml") {
                            let file_stem = path
                                .file_stem()
                                .unwrap_or_default()
                                .to_string_lossy()
                                .to_string();
                            let _ = change_tx.send(file_stem);
                        }
                    }
                }
                let _ = &dir_clone; // keep reference alive
            })
            .map_err(|e| AppError::Anyhow(anyhow::anyhow!("watcher error: {e}")))?;

        watcher
            .watch(&dir, RecursiveMode::NonRecursive)
            .map_err(|e| AppError::Anyhow(anyhow::anyhow!("watch error: {e}")))?;

        self.watcher = Some(watcher);
        Ok(())
    }
}
