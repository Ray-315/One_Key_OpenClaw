import { invoke } from "@tauri-apps/api/core";
import type { DiagnosticReport, TaskGraphData } from "./types";

/** Diagnose a failed step and return fix suggestions. */
export async function diagnoseStepError(
  taskId: string,
  stepId: string,
  rawError: string
): Promise<DiagnosticReport> {
  return invoke<DiagnosticReport>("diagnose_step_error", {
    taskId,
    stepId,
    rawError,
  });
}

/** Return the DAG graph data (nodes + edges) for a recipe. */
export async function getRecipeGraph(
  recipeId: string
): Promise<TaskGraphData> {
  return invoke<TaskGraphData>("get_recipe_graph", { recipeId });
}
