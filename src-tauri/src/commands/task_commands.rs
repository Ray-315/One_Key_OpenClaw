use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use tauri::{AppHandle, State};

use crate::error::AppError;
use crate::state::AppState;
use crate::task::engine::{run_task_executor, Task, TaskStatus};
use crate::task::state_machine::{control_channel, TaskControl};
use crate::task::step::TaskStep;

fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

fn new_task_id() -> String {
    uuid::Uuid::new_v4().to_string()
}

/// Start a task for the given recipe, optionally overriding recipe vars.
/// Returns the task ID.
#[tauri::command]
pub async fn start_task(
    app: AppHandle,
    state: State<'_, AppState>,
    recipe_id: String,
    #[allow(unused_variables)]
    vars: HashMap<String, String>,
) -> Result<String, AppError> {
    // Look up the recipe.
    let recipe = {
        let registry = state
            .recipes
            .lock()
            .map_err(|e| AppError::Anyhow(anyhow::anyhow!("lock poisoned: {e}")))?;
        registry
            .get(&recipe_id)
            .cloned()
            .ok_or_else(|| AppError::RecipeNotFound { recipe_id: recipe_id.clone() })?
    };

    let task_id = new_task_id();

    // Build the initial TaskStep list from the recipe.
    let steps: Vec<TaskStep> = recipe
        .steps
        .iter()
        .map(|rs| {
            let max_retries = rs.retry.as_ref().map(|r| r.max_attempts).unwrap_or(0);
            let mut s = TaskStep::new(&rs.id, &rs.name, max_retries);
            s.description = rs.description.clone();
            s
        })
        .collect();

    let task = Task {
        id: task_id.clone(),
        name: recipe.name.clone(),
        recipe_id: recipe.id.clone(),
        status: TaskStatus::Idle,
        steps,
        created_at: now_ms(),
        started_at: None,
        finished_at: None,
        progress: 0.0,
        error_summary: None,
    };

    let task_arc = Arc::new(Mutex::new(task));
    let (ctrl_tx, ctrl_rx) = control_channel();

    // Register task and control sender.
    {
        let mut tasks = state
            .tasks
            .lock()
            .map_err(|e| AppError::Anyhow(anyhow::anyhow!("lock poisoned: {e}")))?;
        tasks.insert(task_id.clone(), task_arc.clone());
    }
    {
        let mut controls = state
            .task_controls
            .lock()
            .map_err(|e| AppError::Anyhow(anyhow::anyhow!("lock poisoned: {e}")))?;
        controls.insert(task_id.clone(), ctrl_tx);
    }

    // Spawn the executor in a background task.
    let app_clone = app.clone();
    let vars_clone = vars.clone();
    tokio::spawn(async move {
        run_task_executor(app_clone, task_arc, recipe, vars_clone, ctrl_rx).await;
    });

    Ok(task_id)
}

/// Pause a running task (takes effect after the current step finishes if mid-step).
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

/// Cancel a running or paused task.
#[tauri::command]
pub async fn cancel_task(
    state: State<'_, AppState>,
    task_id: String,
) -> Result<(), AppError> {
    send_control(&state, &task_id, TaskControl::Cancel).await
}

async fn send_control(
    state: &State<'_, AppState>,
    task_id: &str,
    ctrl: TaskControl,
) -> Result<(), AppError> {
    let tx = {
        let controls = state
            .task_controls
            .lock()
            .map_err(|e| AppError::Anyhow(anyhow::anyhow!("lock poisoned: {e}")))?;
        controls
            .get(task_id)
            .cloned()
            .ok_or_else(|| AppError::TaskNotFound { task_id: task_id.into() })?
    };
    tx.send(ctrl)
        .await
        .map_err(|_| AppError::InvalidTaskControl {
            message: format!("task {task_id} is no longer running"),
        })
}

/// Return a snapshot of a task by ID.
#[tauri::command]
pub fn get_task(
    state: State<'_, AppState>,
    task_id: String,
) -> Result<Task, AppError> {
    let tasks = state
        .tasks
        .lock()
        .map_err(|e| AppError::Anyhow(anyhow::anyhow!("lock poisoned: {e}")))?;
    let arc = tasks
        .get(&task_id)
        .ok_or_else(|| AppError::TaskNotFound { task_id: task_id.clone() })?;
    let task = arc
        .lock()
        .map_err(|e| AppError::Anyhow(anyhow::anyhow!("task lock poisoned: {e}")))?
        .clone();
    Ok(task)
}

/// List all tasks (most recent first).
#[tauri::command]
pub fn list_tasks(state: State<'_, AppState>) -> Result<Vec<Task>, AppError> {
    let tasks = state
        .tasks
        .lock()
        .map_err(|e| AppError::Anyhow(anyhow::anyhow!("lock poisoned: {e}")))?;
    let mut list: Vec<Task> = tasks
        .values()
        .filter_map(|arc| arc.lock().ok().map(|t| t.clone()))
        .collect();
    list.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    Ok(list)
}
