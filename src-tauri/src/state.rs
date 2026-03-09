use std::sync::Mutex;

use crate::env::prober::EnvItem;

pub struct AppState {
    pub env_cache: Mutex<Vec<EnvItem>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            env_cache: Mutex::new(Vec::new()),
        }
    }
}
