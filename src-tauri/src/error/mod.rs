use serde::Serialize;

#[derive(Debug, thiserror::Error)]
#[allow(dead_code)]
pub enum AppError {
    #[error("Environment probe failed: {env_id} - {message}")]
    EnvProbeError { env_id: String, message: String },

    #[error("Step execution failed: {step_id} (exit code {exit_code:?}) — {stderr}")]
    StepExecutionError {
        step_id: String,
        exit_code: Option<i32>,
        stderr: String,
    },

    #[error("Recipe parse error: {path} — {message}")]
    RecipeParseError { path: String, message: String },

    #[error("Recipe validation failed: {issues:?}")]
    RecipeValidationError { issues: Vec<String> },

    #[error("Invalid state transition: {from} + {event}")]
    InvalidStateTransition { from: String, event: String },

    #[error("Task not found: {task_id}")]
    TaskNotFound { task_id: String },

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("System error: {0}")]
    Anyhow(#[from] anyhow::Error),
}

impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

