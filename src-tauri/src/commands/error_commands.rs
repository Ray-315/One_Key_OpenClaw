use tauri::State;

use crate::error::engine::ErrorDiagnosticEngine;
use crate::error::{AppError, DiagnosticReport};
use crate::state::AppState;
use crate::task::graph::TaskGraphData;

/// Diagnose a failed step and return a [`DiagnosticReport`] with
/// matched rule and fix suggestions.
#[tauri::command]
pub fn diagnose_step_error(
    _state: State<'_, AppState>,
    task_id: String,
    step_id: String,
    raw_error: String,
) -> Result<DiagnosticReport, AppError> {
    let engine = ErrorDiagnosticEngine::with_builtins();
    Ok(engine.diagnose(&task_id, &step_id, &raw_error))
}

/// Return the DAG graph data for a recipe (nodes + edges) for use with
/// the React Flow visualisation on the frontend.
#[tauri::command]
pub fn get_recipe_graph(
    state: State<'_, AppState>,
    recipe_id: String,
) -> Result<TaskGraphData, AppError> {
    let registry = state
        .recipes
        .lock()
        .map_err(|e| AppError::Anyhow(anyhow::anyhow!("lock poisoned: {e}")))?;

    let recipe = registry
        .get(&recipe_id)
        .ok_or_else(|| AppError::RecipeNotFound {
            recipe_id: recipe_id.clone(),
        })?;

    use crate::task::graph::TaskGraph;
    let graph = TaskGraph::build(&recipe.steps)?;
    Ok(graph.to_graph_data(&recipe.steps))
}
