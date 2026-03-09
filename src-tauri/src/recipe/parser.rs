use std::collections::HashMap;

use crate::error::AppError;
use crate::recipe::schema::Recipe;
use crate::task::engine::{Task, TaskStatus};
use crate::task::step::{StepAction, StepStatus, TaskStep};

/// Parse a TOML string into a [`Recipe`].
pub fn parse_toml(source: &str) -> Result<Recipe, AppError> {
    toml::from_str(source).map_err(|e| AppError::RecipeParseError {
        path: "<inline>".to_string(),
        message: e.to_string(),
    })
}

/// Parse a TOML file from the file-system into a [`Recipe`].
pub fn parse_file(path: &str) -> Result<Recipe, AppError> {
    let source = std::fs::read_to_string(path).map_err(|e| AppError::RecipeParseError {
        path: path.to_string(),
        message: format!("Cannot read file: {e}"),
    })?;
    toml::from_str(&source).map_err(|e| AppError::RecipeParseError {
        path: path.to_string(),
        message: e.to_string(),
    })
}

fn now_millis() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

/// Expand `${VAR}` placeholders in a string using the provided variable map.
pub fn expand_vars(s: &str, vars: &HashMap<String, String>) -> String {
    let mut result = s.to_string();
    for (k, v) in vars {
        result = result.replace(&format!("${{{k}}}"), v);
    }
    result
}

impl Recipe {
    /// Convert this recipe into an executable [`Task`], substituting variables.
    pub fn into_task(self, extra_vars: HashMap<String, String>) -> Result<Task, AppError> {
        // Merge recipe default vars with caller-supplied vars (caller wins).
        let mut vars = self.vars.clone();
        vars.extend(extra_vars);

        // Convert each RecipeStep → TaskStep.
        let steps: Vec<TaskStep> = self
            .steps
            .iter()
            .map(|rs| recipe_step_to_task_step(rs, &vars))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Task {
            id: uuid::Uuid::new_v4().to_string(),
            name: expand_vars(&self.name, &vars),
            recipe_id: self.id,
            status: TaskStatus::Idle,
            steps,
            created_at: now_millis(),
            started_at: None,
            finished_at: None,
            progress: 0.0,
            error_summary: None,
        })
    }
}

fn recipe_step_to_task_step(
    rs: &crate::recipe::schema::RecipeStep,
    vars: &HashMap<String, String>,
) -> Result<TaskStep, AppError> {
    use crate::recipe::schema::RecipeStepAction;

    let action = match &rs.action {
        RecipeStepAction::Shell { command, args, env } => StepAction::Shell {
            command: expand_vars(command, vars),
            args: args.iter().map(|a| expand_vars(a, vars)).collect(),
            env: env
                .iter()
                .map(|(k, v)| (k.clone(), expand_vars(v, vars)))
                .collect(),
        },
        RecipeStepAction::PackageInstall { manager, packages } => StepAction::PackageInstall {
            manager: manager.clone(),
            packages: packages.clone(),
        },
        RecipeStepAction::EnvCheck { env_id } => StepAction::EnvCheck {
            env_id: env_id.clone(),
        },
        RecipeStepAction::Download { url, dest } => StepAction::Download {
            url: expand_vars(url, vars),
            dest: expand_vars(dest, vars),
        },
        RecipeStepAction::Extract { src, dest } => StepAction::Extract {
            src: expand_vars(src, vars),
            dest: expand_vars(dest, vars),
        },
    };

    Ok(TaskStep {
        id: rs.id.clone(),
        name: expand_vars(&rs.name, vars),
        description: rs.description.as_deref().map(|d| expand_vars(d, vars)),
        action,
        depends_on: rs.depends_on.clone(),
        status: StepStatus::Pending,
        started_at: None,
        finished_at: None,
        exit_code: None,
        retry_count: 0,
        max_retries: rs
            .retry
            .as_ref()
            .map(|r| r.max_attempts.saturating_sub(1))
            .unwrap_or(0),
    })
}
