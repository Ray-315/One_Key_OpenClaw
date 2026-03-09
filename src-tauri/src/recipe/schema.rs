use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// Top-level recipe document.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Recipe {
    /// Format version, currently "1".
    pub version: String,
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub author: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    /// Target platforms: ["macos", "windows", "linux"] or ["*"].
    #[serde(default)]
    pub platforms: Vec<String>,
    /// Environment prerequisites.
    #[serde(default)]
    pub env_requirements: Vec<EnvRequirement>,
    /// Step definitions.
    #[serde(default)]
    pub steps: Vec<RecipeStep>,
    /// Recipe-level variables.
    #[serde(default)]
    pub vars: HashMap<String, String>,
    #[serde(default)]
    pub metadata: RecipeMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvRequirement {
    pub env_id: String,
    /// Semver constraint, e.g. ">=18.0.0".
    pub version: Option<String>,
    #[serde(default)]
    pub optional: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecipeStep {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub action: RecipeStepAction,
    #[serde(default)]
    pub depends_on: Vec<String>,
    /// Simple variable-expansion condition string (not evaluated in Phase 2).
    pub condition: Option<String>,
    pub retry: Option<RetryConfig>,
    pub timeout_secs: Option<u64>,
    #[serde(default)]
    pub on_error: OnErrorStrategy,
}

/// The action a step performs, serialised as a flattened TOML table.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum RecipeStepAction {
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

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum OnErrorStrategy {
    #[default]
    Fail,
    Skip,
    Retry,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    pub max_attempts: u8,
    #[serde(default)]
    pub delay_secs: u64,
    #[serde(default)]
    pub backoff: BackoffStrategy,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum BackoffStrategy {
    #[default]
    Fixed,
    Exponential,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RecipeMetadata {
    pub created_at: Option<String>,
    pub source_url: Option<String>,
    pub checksum: Option<String>,
}

/// A single validation issue found during recipe validation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationIssue {
    pub field: String,
    pub message: String,
    pub severity: ValidationSeverity,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidationSeverity {
    Error,
    Warning,
}

impl Recipe {
    /// Validate the recipe and return any issues found.
    pub fn validate(&self) -> Vec<ValidationIssue> {
        let mut issues = Vec::new();

        if self.version != "1" {
            issues.push(ValidationIssue {
                field: "version".to_string(),
                message: format!("Unsupported recipe version '{}'; expected '1'", self.version),
                severity: ValidationSeverity::Error,
            });
        }

        if self.id.is_empty() {
            issues.push(ValidationIssue {
                field: "id".to_string(),
                message: "Recipe id must not be empty".to_string(),
                severity: ValidationSeverity::Error,
            });
        }

        if self.name.is_empty() {
            issues.push(ValidationIssue {
                field: "name".to_string(),
                message: "Recipe name must not be empty".to_string(),
                severity: ValidationSeverity::Error,
            });
        }

        if self.steps.is_empty() {
            issues.push(ValidationIssue {
                field: "steps".to_string(),
                message: "Recipe must contain at least one step".to_string(),
                severity: ValidationSeverity::Warning,
            });
        }

        // Check for duplicate step ids.
        let mut seen_ids = std::collections::HashSet::new();
        for step in &self.steps {
            if !seen_ids.insert(&step.id) {
                issues.push(ValidationIssue {
                    field: format!("steps[{}].id", step.id),
                    message: format!("Duplicate step id '{}'", step.id),
                    severity: ValidationSeverity::Error,
                });
            }
        }

        // Check depends_on references exist.
        for step in &self.steps {
            for dep in &step.depends_on {
                if !self.steps.iter().any(|s| &s.id == dep) {
                    issues.push(ValidationIssue {
                        field: format!("steps[{}].dependsOn", step.id),
                        message: format!(
                            "Step '{}' depends on unknown step '{}'",
                            step.id, dep
                        ),
                        severity: ValidationSeverity::Error,
                    });
                }
            }
        }

        issues
    }
}
