use std::collections::HashMap;
use std::sync::Mutex;

use crate::env::prober::EnvItem;
use crate::recipe::RecipeRegistry;
use crate::task::engine::TaskHandle;

pub struct AppState {
    pub env_cache: Mutex<Vec<EnvItem>>,
    pub recipe_registry: RecipeRegistry,
    /// Active task handles keyed by task_id.
    pub tasks: Mutex<HashMap<String, TaskHandle>>,
}

impl Default for AppState {
    fn default() -> Self {
        let registry = RecipeRegistry::default();
        if let Err(e) = registry.load_builtin() {
            eprintln!("[AppState] Failed to load built-in recipes: {e}");
        }
        Self {
            env_cache: Mutex::new(Vec::new()),
            recipe_registry: registry,
            tasks: Mutex::new(HashMap::new()),
        }
    }
}

