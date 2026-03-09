use serde::{Deserialize, Serialize};

/// Runtime status of a single step.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum StepStatus {
    /// Waiting to be scheduled.
    Pending,
    /// Blocked on a dependency (future DAG use).
    Waiting,
    /// Currently executing.
    Running,
    /// Completed successfully.
    Success,
    /// Failed with an error message.
    Failed { error: String },
    /// Skipped due to on_error = skip.
    Skipped,
    /// Cancelled by the user.
    Cancelled,
}

/// Lightweight runtime view of a recipe step (frontend-visible).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskStep {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub status: StepStatus,
    pub started_at: Option<u64>,
    pub finished_at: Option<u64>,
    pub exit_code: Option<i32>,
    /// Number of times this step has been retried.
    pub retry_count: u8,
    pub max_retries: u8,
}

impl TaskStep {
    /// Create a new step in Pending state from basic info.
    pub fn new(id: impl Into<String>, name: impl Into<String>, max_retries: u8) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: None,
            status: StepStatus::Pending,
            started_at: None,
            finished_at: None,
            exit_code: None,
            retry_count: 0,
            max_retries,
        }
    }
}
