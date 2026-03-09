import { invoke } from "@tauri-apps/api/core";
import type { Task } from "./types";

/** Start a task for the given recipe. Returns the new task ID. */
export async function startTask(
  recipeId: string,
  vars: Record<string, string> = {}
): Promise<string> {
  return invoke<string>("start_task", { recipeId, vars });
}

/** Pause a running task. */
export async function pauseTask(taskId: string): Promise<void> {
  return invoke("pause_task", { taskId });
}

/** Resume a paused task. */
export async function resumeTask(taskId: string): Promise<void> {
  return invoke("resume_task", { taskId });
}

/** Cancel a running or paused task. */
export async function cancelTask(taskId: string): Promise<void> {
  return invoke("cancel_task", { taskId });
}

/** Get the current snapshot of a task by ID. */
export async function getTask(taskId: string): Promise<Task> {
  return invoke<Task>("get_task", { taskId });
}

/** List all tasks, most recent first. */
export async function listTasks(): Promise<Task[]> {
  return invoke<Task[]>("list_tasks");
}
