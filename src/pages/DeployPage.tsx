import { useEffect, useCallback, useState } from "react";
import { useRecipeStore } from "../store/recipeStore";
import { useTaskStore } from "../store/taskStore";
import { StepList } from "../components/task/StepList";
import { ProgressBar } from "../components/task/ProgressBar";
import { useTauriEvent } from "../hooks/useTauriEvent";
import type {
  TaskStatusEvent,
  TaskProgressEvent,
  TaskStep,
  TaskStatus,
} from "../ipc/types";

function taskStatusLabel(status: TaskStatus): string {
  switch (status) {
    case "idle":
      return "就绪";
    case "running":
      return "执行中";
    case "paused":
      return "已暂停";
    case "success":
      return "成功";
    case "failed":
      return "失败";
    case "cancelled":
      return "已取消";
  }
}

function taskStatusClass(status: TaskStatus): string {
  switch (status) {
    case "running":
      return "text-blue-400";
    case "paused":
      return "text-[var(--color-warning)]";
    case "success":
      return "text-[var(--color-success)]";
    case "failed":
      return "text-[var(--color-error)]";
    case "cancelled":
      return "text-[var(--color-text-muted)]";
    default:
      return "text-[var(--color-text-muted)]";
  }
}

export function DeployPage() {
  const { recipes, loading: recipesLoading, loadRecipes } = useRecipeStore();
  const {
    tasks,
    activeTaskId,
    loading: taskLoading,
    error,
    start,
    pause,
    resume,
    cancel,
    applyStatusEvent,
    applyProgressEvent,
    applyStepUpdate,
  } = useTaskStore();

  const activeTask = activeTaskId ? tasks[activeTaskId] : null;

  // Controlled recipe selection.
  const [selectedRecipeId, setSelectedRecipeId] = useState<string>("");

  // Load recipes on mount.
  useEffect(() => {
    loadRecipes();
  }, [loadRecipes]);

  // Default to first recipe once loaded.
  useEffect(() => {
    if (recipes.length > 0 && !selectedRecipeId) {
      setSelectedRecipeId(recipes[0].id);
    }
  }, [recipes, selectedRecipeId]);

  // Subscribe to Tauri task events.
  const handleStatus = useCallback(
    (e: TaskStatusEvent) => applyStatusEvent(e),
    [applyStatusEvent]
  );
  const handleProgress = useCallback(
    (e: TaskProgressEvent) => applyProgressEvent(e),
    [applyProgressEvent]
  );
  const handleStepUpdate = useCallback(
    (step: TaskStep) => applyStepUpdate(step),
    [applyStepUpdate]
  );

  useTauriEvent<TaskStatusEvent>("task://status", handleStatus);
  useTauriEvent<TaskProgressEvent>("task://progress", handleProgress);
  useTauriEvent<TaskStep>("task://step-update", handleStepUpdate);

  const handleStart = async () => {
    if (!selectedRecipeId) return;
    try {
      await start(selectedRecipeId);
    } catch {
      // error is handled in the store
    }
  };

  const isRunning = activeTask?.status === "running";
  const isPaused = activeTask?.status === "paused";
  const isFinished =
    activeTask?.status === "success" ||
    activeTask?.status === "failed" ||
    activeTask?.status === "cancelled";

  return (
    <div className="flex h-full flex-col">
      {/* Header */}
      <div className="flex items-center justify-between border-b border-[var(--color-border)] px-6 py-4">
        <h2 className="text-lg font-bold">一键部署</h2>

        <div className="flex items-center gap-3">
          {/* Recipe selector */}
          {!activeTask || isFinished ? (
            <select
              className="rounded-md border border-[var(--color-border)] bg-[var(--color-surface)] px-3 py-2 text-sm text-[var(--color-text)] focus:outline-none focus:ring-1 focus:ring-[var(--color-primary)]"
              value={selectedRecipeId}
              onChange={(e) => setSelectedRecipeId(e.target.value)}
              disabled={recipesLoading}
            >
              {recipes.length === 0 ? (
                <option value="">暂无配方</option>
              ) : (
                recipes.map((r) => (
                  <option key={r.id} value={r.id}>
                    {r.name}
                  </option>
                ))
              )}
            </select>
          ) : null}

          {/* Action buttons */}
          {!activeTask || isFinished ? (
            <button
              onClick={handleStart}
              disabled={taskLoading || !selectedRecipeId}
              className="rounded-md bg-[var(--color-primary)] px-4 py-2 text-sm font-medium text-white transition-colors hover:bg-[var(--color-primary-hover)] disabled:opacity-50"
            >
              {taskLoading ? "启动中..." : "开始部署"}
            </button>
          ) : (
            <>
              {isRunning && (
                <button
                  onClick={pause}
                  className="rounded-md border border-[var(--color-border)] px-3 py-2 text-sm text-[var(--color-text)] hover:bg-[var(--color-surface-hover)]"
                >
                  暂停
                </button>
              )}
              {isPaused && (
                <button
                  onClick={resume}
                  className="rounded-md bg-[var(--color-primary)] px-3 py-2 text-sm font-medium text-white hover:bg-[var(--color-primary-hover)]"
                >
                  继续
                </button>
              )}
              <button
                onClick={cancel}
                className="rounded-md border border-red-500/40 px-3 py-2 text-sm text-[var(--color-error)] hover:bg-red-500/10"
              >
                取消
              </button>
            </>
          )}
        </div>
      </div>

      {/* Error banner */}
      {error && (
        <div className="mx-6 mt-4 rounded-md bg-red-500/10 px-4 py-3 text-sm text-red-400">
          {error}
        </div>
      )}

      {/* Content */}
      <div className="flex-1 overflow-auto p-6 space-y-6">
        {/* No task started yet */}
        {!activeTask && !taskLoading && (
          <div className="flex flex-col items-center justify-center py-16 text-[var(--color-text-muted)]">
            <span className="text-5xl mb-4">🚀</span>
            <p className="text-base">选择配方，点击「开始部署」</p>
          </div>
        )}

        {activeTask && (
          <>
            {/* Task status + progress */}
            <div className="rounded-lg border border-[var(--color-border)] bg-[var(--color-surface)] p-4 space-y-3">
              <div className="flex items-center justify-between">
                <span className="text-sm font-semibold text-[var(--color-text)]">
                  {activeTask.name}
                </span>
                <span
                  className={`text-sm font-medium ${taskStatusClass(activeTask.status)}`}
                >
                  {taskStatusLabel(activeTask.status)}
                </span>
              </div>

              <ProgressBar value={activeTask.progress} />

              <p className="text-xs text-[var(--color-text-muted)]">
                总进度 {Math.round(activeTask.progress)}%
                {activeTask.errorSummary && (
                  <span className="ml-2 text-[var(--color-error)]">
                    — {activeTask.errorSummary}
                  </span>
                )}
              </p>
            </div>

            {/* Step list */}
            <div>
              <h3 className="mb-3 text-sm font-semibold text-[var(--color-text-muted)] uppercase tracking-wide">
                步骤进度
              </h3>
              <StepList steps={activeTask.steps} />
            </div>
          </>
        )}
      </div>
    </div>
  );
}
