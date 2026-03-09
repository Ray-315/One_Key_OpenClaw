import { useCallback } from "react";
import { useTauriEvent } from "./useTauriEvent";
import { useTaskStore } from "../store/taskStore";
import { startTask, pauseTask, resumeTask, cancelTask } from "../ipc/taskApi";
import type {
  TaskProgressEvent,
  TaskStatusEvent,
  TaskStep,
} from "../ipc/types";

/**
 * Subscribes to all task-related Tauri events and keeps the task store
 * up to date. Also exposes action helpers for controlling tasks.
 */
export function useTaskRunner() {
  const { updateTaskStatus, updateTaskProgress, setActiveTask } =
    useTaskStore();

  // task://progress
  useTauriEvent<TaskProgressEvent>(
    "task://progress",
    useCallback(
      ({ taskId, progress }) => {
        updateTaskProgress(taskId, progress);
      },
      [updateTaskProgress]
    )
  );

  // task://status
  useTauriEvent<TaskStatusEvent>(
    "task://status",
    useCallback(
      ({ taskId, status, errorSummary }) => {
        updateTaskStatus(taskId, status, errorSummary);
      },
      [updateTaskStatus]
    )
  );

  // task://step-update
  useTauriEvent<TaskStep & { taskId?: string }>(
    "task://step-update",
    useCallback(
      (step) => {
        // Find which task owns this step and update its steps array.
        useTaskStore.setState((state) => {
          const updated = { ...state.tasks };
          for (const [tid, task] of Object.entries(updated)) {
            const idx = task.steps.findIndex((s) => s.id === step.id);
            if (idx >= 0) {
              const steps = [...task.steps];
              steps[idx] = step;
              updated[tid] = { ...task, steps };
              break;
            }
          }
          return { tasks: updated };
        });
      },
      []
    )
  );

  const run = useCallback(
    async (recipeId: string, vars: Record<string, string> = {}) => {
      const taskId = await startTask(recipeId, vars);
      setActiveTask(taskId);
      return taskId;
    },
    [setActiveTask]
  );

  const pause = useCallback(async (taskId: string) => {
    await pauseTask(taskId);
  }, []);

  const resume = useCallback(async (taskId: string) => {
    await resumeTask(taskId);
  }, []);

  const cancel = useCallback(async (taskId: string) => {
    await cancelTask(taskId);
  }, []);

  return { run, pause, resume, cancel };
}
