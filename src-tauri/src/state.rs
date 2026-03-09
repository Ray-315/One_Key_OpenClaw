use std::sync::Mutex;

use crate::env::prober::EnvItem;

pub struct AppState {
    pub env_cache: Mutex<Vec<EnvItem>>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            env_cache: Mutex::new(Vec::new()),
        }
    }
}
