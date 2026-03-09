use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use tokio::sync::mpsc;

use crate::env::prober::EnvItem;
use crate::plugin::manager::PluginManager;
use crate::recipe::registry::RecipeRegistry;
use crate::task::engine::Task;
use crate::task::state_machine::TaskControl;

pub struct AppState {
    pub env_cache: Mutex<Vec<EnvItem>>,
    pub recipes: Mutex<RecipeRegistry>,
    pub tasks: Mutex<HashMap<String, Arc<Mutex<Task>>>>,
    pub task_controls: Mutex<HashMap<String, mpsc::Sender<TaskControl>>>,
    pub plugins: Mutex<PluginManager>,
}

impl Default for AppState {
    fn default() -> Self {
        // Default plugin directory: alongside the executable in a `plugins/` folder.
        // Falls back to a relative `plugins/` path if the executable path
        // cannot be determined (unlikely in practice).
        let plugin_dir = std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|d| d.join("plugins")))
            .unwrap_or_else(|| {
                eprintln!("[AppState] Could not determine exe path, using relative plugins/");
                std::path::PathBuf::from("plugins")
            });

        let mut plugin_mgr = PluginManager::new(plugin_dir);
        // Best-effort scan at startup.
        let _ = plugin_mgr.scan_plugins();

        Self {
            env_cache: Mutex::new(Vec::new()),
            recipes: Mutex::new(RecipeRegistry::with_builtins()),
            tasks: Mutex::new(HashMap::new()),
            task_controls: Mutex::new(HashMap::new()),
            plugins: Mutex::new(plugin_mgr),
        }
    }
}
