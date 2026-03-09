use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

/// Control messages sent from Tauri commands to the running executor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TaskControl {
    Pause,
    Resume,
    Cancel,
}

/// Events that drive task-level state transitions.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[allow(dead_code)]
pub enum TaskEvent {
    Start,
    Pause,
    Resume,
    Complete,
    Fail,
    Cancel,
}

/// Create a bounded control channel for a task (buffer = 4 messages).
pub fn control_channel() -> (mpsc::Sender<TaskControl>, mpsc::Receiver<TaskControl>) {
    mpsc::channel(4)
}
