use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// A single executable step within a task.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskStep {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub action: StepAction,
    pub depends_on: Vec<String>,
    pub status: StepStatus,
    pub started_at: Option<u64>,
    pub finished_at: Option<u64>,
    pub exit_code: Option<i32>,
    pub retry_count: u8,
    pub max_retries: u8,
}

/// What a step does when executed.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum StepAction {
    Shell {
        command: String,
        #[serde(default)]
        args: Vec<String>,
        #[serde(default)]
        env: HashMap<String, String>,
    },
    PackageInstall {
        manager: String,
        packages: Vec<String>,
    },
    EnvCheck {
        env_id: String,
    },
    Download {
        url: String,
        dest: String,
    },
    Extract {
        src: String,
        dest: String,
    },
}

/// Lifecycle status of a single step.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum StepStatus {
    Pending,
    Waiting,
    Running,
    Success,
    Failed { error: String },
    Skipped,
    Cancelled,
}
