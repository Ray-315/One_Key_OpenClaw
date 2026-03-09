use std::collections::HashMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Recipe top-level
// ---------------------------------------------------------------------------

/// A declarative deployment recipe.
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
    /// Environment pre-checks.
    #[serde(default, alias = "env_requirements")]
    pub env_requirements: Vec<EnvRequirement>,
    /// Ordered step definitions.
    pub steps: Vec<RecipeStep>,
    /// Recipe-level variables for substitution.
    #[serde(default)]
    pub vars: HashMap<String, String>,
}

/// An environment item required by the recipe.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EnvRequirement {
    #[serde(alias = "env_id")]
    pub env_id: String,
    /// Optional semver constraint, e.g. ">=18.0.0".
    pub version: Option<String>,
    #[serde(default)]
    pub optional: bool,
}

// ---------------------------------------------------------------------------
// Recipe step
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecipeStep {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub action: StepAction,
    /// Step IDs this step depends on (for DAG; ignored in Phase-2 serial mode).
    #[serde(default, alias = "depends_on")]
    pub depends_on: Vec<String>,
    /// Condition expression (future use).
    pub condition: Option<String>,
    pub retry: Option<RetryConfig>,
    #[serde(alias = "timeout_secs")]
    pub timeout_secs: Option<u64>,
    #[serde(default, alias = "on_error")]
    pub on_error: OnErrorStrategy,
}

// ---------------------------------------------------------------------------
// Step action
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum StepAction {
    /// Execute a shell command.
    Shell {
        command: String,
        #[serde(default)]
        args: Vec<String>,
        #[serde(default)]
        env: HashMap<String, String>,
    },
    /// Install packages via a package manager.
    PackageInstall {
        manager: PackageManager,
        packages: Vec<String>,
    },
    /// Assert that an environment item is available.
    EnvCheck {
        #[serde(alias = "env_id")]
        env_id: String,
    },
    /// Download a file.
    Download { url: String, dest: PathBuf },
    /// Extract an archive.
    Extract { src: PathBuf, dest: PathBuf },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum PackageManager {
    Npm,
    Pip,
    Cargo,
    Brew,
    Apt,
    Winget,
}

// ---------------------------------------------------------------------------
// Retry configuration
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RetryConfig {
    #[serde(alias = "max_attempts")]
    pub max_attempts: u8,
    #[serde(default = "default_delay_secs", alias = "delay_secs")]
    pub delay_secs: u64,
    #[serde(default)]
    pub backoff: BackoffStrategy,
}

fn default_delay_secs() -> u64 {
    3
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum BackoffStrategy {
    #[default]
    Fixed,
    Exponential,
}

// ---------------------------------------------------------------------------
// Error strategy
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum OnErrorStrategy {
    #[default]
    Fail,
    Skip,
    Retry,
}

// ---------------------------------------------------------------------------
// Recipe validation
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidationIssue {
    pub field: String,
    pub message: String,
    pub severity: IssueSeverity,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum IssueSeverity {
    Error,
    Warning,
}

/// Validate a recipe and return any issues found.
pub fn validate_recipe(recipe: &Recipe) -> Vec<ValidationIssue> {
    let mut issues = Vec::new();

    if recipe.version.is_empty() {
        issues.push(ValidationIssue {
            field: "version".into(),
            message: "version must not be empty".into(),
            severity: IssueSeverity::Error,
        });
    }
    if recipe.id.is_empty() {
        issues.push(ValidationIssue {
            field: "id".into(),
            message: "id must not be empty".into(),
            severity: IssueSeverity::Error,
        });
    }
    if recipe.name.is_empty() {
        issues.push(ValidationIssue {
            field: "name".into(),
            message: "name must not be empty".into(),
            severity: IssueSeverity::Error,
        });
    }
    if recipe.steps.is_empty() {
        issues.push(ValidationIssue {
            field: "steps".into(),
            message: "recipe must have at least one step".into(),
            severity: IssueSeverity::Warning,
        });
    }

    // Ensure step IDs are unique.
    let mut seen_ids = std::collections::HashSet::new();
    let known_ids: std::collections::HashSet<&str> =
        recipe.steps.iter().map(|step| step.id.as_str()).collect();
    for (idx, step) in recipe.steps.iter().enumerate() {
        if step.id.is_empty() {
            issues.push(ValidationIssue {
                field: format!("steps[{idx}].id"),
                message: "step id must not be empty".into(),
                severity: IssueSeverity::Error,
            });
        }
        if !seen_ids.insert(step.id.clone()) {
            issues.push(ValidationIssue {
                field: format!("steps[{idx}].id"),
                message: format!("duplicate step id: {}", step.id),
                severity: IssueSeverity::Error,
            });
        }
        if step.name.trim().is_empty() {
            issues.push(ValidationIssue {
                field: format!("steps[{idx}].name"),
                message: "step name must not be empty".into(),
                severity: IssueSeverity::Error,
            });
        }

        for dep_id in &step.depends_on {
            if dep_id == &step.id {
                issues.push(ValidationIssue {
                    field: format!("steps[{idx}].dependsOn"),
                    message: "step cannot depend on itself".into(),
                    severity: IssueSeverity::Error,
                });
            } else if !known_ids.contains(dep_id.as_str()) {
                issues.push(ValidationIssue {
                    field: format!("steps[{idx}].dependsOn"),
                    message: format!("unknown dependency step id: {dep_id}"),
                    severity: IssueSeverity::Error,
                });
            }
        }

        if step
            .retry
            .as_ref()
            .is_some_and(|retry| retry.max_attempts == 0)
        {
            issues.push(ValidationIssue {
                field: format!("steps[{idx}].retry.maxAttempts"),
                message: "retry.maxAttempts must be greater than 0 when retry is configured".into(),
                severity: IssueSeverity::Error,
            });
        }

        match &step.action {
            StepAction::Shell { command, .. } if command.trim().is_empty() => {
                issues.push(ValidationIssue {
                    field: format!("steps[{idx}].action.command"),
                    message: "shell command must not be empty".into(),
                    severity: IssueSeverity::Error,
                });
            }
            StepAction::PackageInstall { packages, .. } if packages.is_empty() => {
                issues.push(ValidationIssue {
                    field: format!("steps[{idx}].action.packages"),
                    message: "package install step must include at least one package".into(),
                    severity: IssueSeverity::Error,
                });
            }
            StepAction::PackageInstall { packages, .. } => {
                for (pkg_idx, package) in packages.iter().enumerate() {
                    if package.trim().is_empty() {
                        issues.push(ValidationIssue {
                            field: format!("steps[{idx}].action.packages[{pkg_idx}]"),
                            message: "package name must not be empty".into(),
                            severity: IssueSeverity::Error,
                        });
                    }
                }
            }
            StepAction::EnvCheck { env_id } if env_id.trim().is_empty() => {
                issues.push(ValidationIssue {
                    field: format!("steps[{idx}].action.envId"),
                    message: "env check step must specify envId".into(),
                    severity: IssueSeverity::Error,
                });
            }
            StepAction::Download { .. } => {
                issues.push(ValidationIssue {
                    field: format!("steps[{idx}].action.type"),
                    message: "download steps are not supported yet".into(),
                    severity: IssueSeverity::Error,
                });
            }
            StepAction::Extract { .. } => {
                issues.push(ValidationIssue {
                    field: format!("steps[{idx}].action.type"),
                    message: "extract steps are not supported yet".into(),
                    severity: IssueSeverity::Error,
                });
            }
            _ => {}
        }
    }

    issues
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_recipe() -> Recipe {
        Recipe {
            version: "1".into(),
            id: "recipe-id".into(),
            name: "Recipe".into(),
            description: None,
            author: None,
            tags: Vec::new(),
            platforms: vec!["linux".into()],
            env_requirements: Vec::new(),
            steps: vec![RecipeStep {
                id: "step-1".into(),
                name: "Step 1".into(),
                description: None,
                action: StepAction::Shell {
                    command: "echo".into(),
                    args: vec!["ok".into()],
                    env: HashMap::new(),
                },
                depends_on: Vec::new(),
                condition: None,
                retry: None,
                timeout_secs: None,
                on_error: OnErrorStrategy::Fail,
            }],
            vars: HashMap::new(),
        }
    }

    #[test]
    fn validate_recipe_reports_unknown_dependencies_and_invalid_actions() {
        let mut recipe = base_recipe();
        recipe.steps[0].depends_on.push("missing-step".into());
        recipe.steps[0].action = StepAction::PackageInstall {
            manager: PackageManager::Npm,
            packages: Vec::new(),
        };

        let issues = validate_recipe(&recipe);

        assert!(issues
            .iter()
            .any(|issue| issue.message.contains("unknown dependency step id")));
        assert!(issues.iter().any(|issue| {
            issue
                .message
                .contains("package install step must include at least one package")
        }));
    }

    #[test]
    fn validate_recipe_rejects_unsupported_download_and_zero_retry() {
        let mut recipe = base_recipe();
        recipe.steps[0].action = StepAction::Download {
            url: "https://example.com/archive.zip".into(),
            dest: PathBuf::from("/tmp/archive.zip"),
        };
        recipe.steps[0].retry = Some(RetryConfig {
            max_attempts: 0,
            delay_secs: 1,
            backoff: BackoffStrategy::Fixed,
        });

        let issues = validate_recipe(&recipe);

        assert!(issues.iter().any(|issue| issue
            .message
            .contains("download steps are not supported yet")));
        assert!(issues.iter().any(|issue| {
            issue
                .message
                .contains("retry.maxAttempts must be greater than 0")
        }));
    }
}
