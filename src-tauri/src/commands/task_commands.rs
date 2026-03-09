use std::collections::HashMap;

use tauri::{AppHandle, State};

use crate::error::AppError;
use crate::state::AppState;
use crate::task::engine::{Task, TaskExecutor};
use crate::task::state_machine::TaskControl;

/// Start a task from a recipe by id.
#[tauri::command]
pub async fn start_task(
    app: AppHandle,
    state: State<'_, AppState>,
    recipe_id: String,
    vars: HashMap<String, String>,
) -> Result<String, AppError> {
    let recipe = state
        .recipe_registry
        .get(&recipe_id)?
        .ok_or_else(|| AppError::Anyhow(anyhow::anyhow!("Recipe '{}' not found", recipe_id)))?;

    let task = recipe.into_task(vars)?;
    let task_id = task.id.clone();

    let executor = TaskExecutor::new(app);
    let handle = executor.spawn(task);

    state
        .tasks
        .lock()
        .map_err(|e| AppError::Anyhow(anyhow::anyhow!("Lock poisoned: {e}")))?
        .insert(task_id.clone(), handle);

    Ok(task_id)
}

/// Pause a running task.
#[tauri::command]
pub async fn pause_task(
    state: State<'_, AppState>,
    task_id: String,
) -> Result<(), AppError> {
    send_control(&state, &task_id, TaskControl::Pause).await
}

/// Resume a paused task.
#[tauri::command]
pub async fn resume_task(
    state: State<'_, AppState>,
    task_id: String,
) -> Result<(), AppError> {
    send_control(&state, &task_id, TaskControl::Resume).await
}

/// Cancel a task.
#[tauri::command]
pub async fn cancel_task(
    state: State<'_, AppState>,
    task_id: String,
) -> Result<(), AppError> {
    send_control(&state, &task_id, TaskControl::Cancel).await
}

/// Get a snapshot of a task.
#[tauri::command]
pub fn get_task(
    state: State<'_, AppState>,
    task_id: String,
) -> Result<Task, AppError> {
    let tasks = state
        .tasks
        .lock()
        .map_err(|e| AppError::Anyhow(anyhow::anyhow!("Lock poisoned: {e}")))?;

    let handle = tasks
        .get(&task_id)
        .ok_or_else(|| AppError::TaskNotFound { task_id: task_id.clone() })?;

    let task = handle
        .task
        .lock()
        .map_err(|e| AppError::Anyhow(anyhow::anyhow!("Lock poisoned: {e}")))?
        .clone();

    Ok(task)
}

/// List all tasks (snapshots).
#[tauri::command]
pub fn list_tasks(state: State<'_, AppState>) -> Result<Vec<Task>, AppError> {
    let tasks = state
        .tasks
        .lock()
        .map_err(|e| AppError::Anyhow(anyhow::anyhow!("Lock poisoned: {e}")))?;

    let list = tasks
        .values()
        .map(|h| {
            h.task
                .lock()
                .map(|t| t.clone())
                .map_err(|e| AppError::Anyhow(anyhow::anyhow!("Lock poisoned: {e}")))
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok(list)
}

// ─── Internal helpers ─────────────────────────────────────────────────────────

async fn send_control(
    state: &State<'_, AppState>,
    task_id: &str,
    control: TaskControl,
) -> Result<(), AppError> {
    let tx = {
        let tasks = state
            .tasks
            .lock()
            .map_err(|e| AppError::Anyhow(anyhow::anyhow!("Lock poisoned: {e}")))?;

        tasks
            .get(task_id)
            .ok_or_else(|| AppError::TaskNotFound {
                task_id: task_id.to_string(),
            })?
            .control_tx
            .clone()
    };

    tx.send(control)
        .await
        .map_err(|e| AppError::Anyhow(anyhow::anyhow!("Failed to send control signal: {e}")))?;

    Ok(())
}
