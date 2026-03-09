import { invoke } from "@tauri-apps/api/core";
import type { Task } from "./types";

/** Start a task from a recipe */
export async function startTask(
  recipeId: string,
  vars: Record<string, string> = {}
): Promise<string> {
  return invoke<string>("start_task", { recipeId, vars });
}

/** Pause a running task */
export async function pauseTask(taskId: string): Promise<void> {
  return invoke<void>("pause_task", { taskId });
}

/** Resume a paused task */
export async function resumeTask(taskId: string): Promise<void> {
  return invoke<void>("resume_task", { taskId });
}

/** Cancel a task */
export async function cancelTask(taskId: string): Promise<void> {
  return invoke<void>("cancel_task", { taskId });
}

/** Get a snapshot of a task */
export async function getTask(taskId: string): Promise<Task> {
  return invoke<Task>("get_task", { taskId });
}

/** List all tasks */
export async function listTasks(): Promise<Task[]> {
  return invoke<Task[]>("list_tasks");
}
