pub mod engine;

use serde::Serialize;

#[derive(Debug, thiserror::Error)]
#[allow(dead_code)]
pub enum AppError {
    #[error("Environment probe failed: {env_id} - {message}")]
    EnvProbeError { env_id: String, message: String },

    #[error("Recipe parse error: {path} - {message}")]
    RecipeParseError { path: String, message: String },

    #[error("Recipe not found: {recipe_id}")]
    RecipeNotFound { recipe_id: String },

    #[error("Step execution failed: {step_id} (exit code {exit_code:?}) - {stderr}")]
    StepExecutionError {
        step_id: String,
        exit_code: Option<i32>,
        stderr: String,
    },

    #[error("Task not found: {task_id}")]
    TaskNotFound { task_id: String },

    #[error("Invalid task control: {message}")]
    InvalidTaskControl { message: String },

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

// ---------------------------------------------------------------------------
// Error diagnostic types
// ---------------------------------------------------------------------------

use serde::Deserialize;

/// Category of a matched error.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ErrorCategory {
    NetworkError,
    PermissionDenied,
    MissingDependency,
    VersionConflict,
    DiskSpace,
    Timeout,
    Unknown,
}

/// An auto-fix action that can be executed.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum FixAction {
    RunCommand { command: String, args: Vec<String> },
    RetryStep { step_id: String },
    InstallEnv { env_id: String },
    OpenUrl { url: String },
}

/// A human-readable suggestion with an optional auto-fix action.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FixSuggestion {
    pub title: String,
    pub description: String,
    pub action: Option<FixAction>,
}

/// A rule that matches a raw error string and provides suggestions.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ErrorRule {
    pub id: String,
    /// Regex pattern matched against the raw error text.
    pub pattern: String,
    pub description: String,
    pub category: ErrorCategory,
    #[serde(default)]
    pub suggestions: Vec<FixSuggestion>,
}

/// Full diagnostic report produced for a failed step.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiagnosticReport {
    pub task_id: String,
    pub step_id: String,
    pub raw_error: String,
    pub matched_rule: Option<ErrorRule>,
    pub suggestions: Vec<FixSuggestion>,
    pub auto_fixable: bool,
}
