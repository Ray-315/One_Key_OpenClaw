use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use tokio::sync::mpsc;

use crate::env::prober::EnvItem;
use crate::recipe::registry::RecipeRegistry;
use crate::task::engine::Task;
use crate::task::state_machine::TaskControl;

pub struct AppState {
    pub env_cache: Mutex<Vec<EnvItem>>,
    pub recipes: Mutex<RecipeRegistry>,
    pub tasks: Mutex<HashMap<String, Arc<Mutex<Task>>>>,
    pub task_controls: Mutex<HashMap<String, mpsc::Sender<TaskControl>>>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            env_cache: Mutex::new(Vec::new()),
            recipes: Mutex::new(RecipeRegistry::with_builtins()),
            tasks: Mutex::new(HashMap::new()),
            task_controls: Mutex::new(HashMap::new()),
        }
    }
}
