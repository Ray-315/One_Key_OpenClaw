import { useEffect, useState, useCallback } from "react";
import { useRecipeStore } from "../store/recipeStore";
import { useTaskStore } from "../store/taskStore";
import { useTaskRunner } from "../hooks/useTaskRunner";
import { useLogStream } from "../hooks/useLogStream";
import { StepList } from "../components/task/StepList";
import { ProgressBar } from "../components/task/ProgressBar";
import { LogTerminal } from "../components/log/LogTerminal";
import { ErrorAlert } from "../components/error/ErrorAlert";
import { diagnoseStepError } from "../ipc/errorApi";
import type { Recipe, DiagnosticReport } from "../ipc/types";

export function DeployPage() {
  const { recipes, loading: recipesLoading, loadRecipes } = useRecipeStore();
  const { tasks, activeTaskId, setActiveTask } = useTaskStore();
  const { run, pause, resume, cancel } = useTaskRunner();

  const [selectedRecipeId, setSelectedRecipeId] = useState<string>("");
  const [error, setError] = useState<string | null>(null);
  const [showLog, setShowLog] = useState(true);
  const [diagnostics, setDiagnostics] = useState<DiagnosticReport | null>(null);

  const activeTask = activeTaskId ? tasks[activeTaskId] : null;

  // Subscribe to log entries for the active task.
  useLogStream(activeTaskId ?? undefined);

  useEffect(() => {
    loadRecipes();
  }, [loadRecipes]);

  // Auto-select first recipe.
  useEffect(() => {
    if (!selectedRecipeId && recipes.length > 0) {
      setSelectedRecipeId(recipes[0].id);
    }
  }, [recipes, selectedRecipeId]);

  const selectedRecipe: Recipe | undefined = recipes.find(
    (r) => r.id === selectedRecipeId
  );

  const isRunning = activeTask?.status === "running";
  const isPaused = activeTask?.status === "paused";
  const isFinished =
    activeTask?.status === "success" ||
    activeTask?.status === "failed" ||
    activeTask?.status === "cancelled";

  async function handleDeploy() {
    if (!selectedRecipeId) return;
    setError(null);
    try {
      const taskId = await run(selectedRecipeId);
      setActiveTask(taskId);
    } catch (e) {
      setError(String(e));
    }
  }

  async function handlePause() {
    if (!activeTaskId) return;
    try {
      await pause(activeTaskId);
    } catch (e) {
      setError(String(e));
    }
  }

  async function handleResume() {
    if (!activeTaskId) return;
    try {
      await resume(activeTaskId);
    } catch (e) {
      setError(String(e));
    }
  }

  async function handleCancel() {
    if (!activeTaskId) return;
    try {
      await cancel(activeTaskId);
    } catch (e) {
      setError(String(e));
    }
  }

  function handleNewDeploy() {
    setActiveTask(null);
    setError(null);
    setDiagnostics(null);
  }

  // Auto-diagnose when a task fails and there's a failed step.
  const runDiagnostics = useCallback(async () => {
    if (!activeTask || activeTask.status !== "failed") return;
    const failedStep = activeTask.steps.find(
      (s) => s.status && typeof s.status === "object" && s.status.type === "failed"
    );
    if (!failedStep) return;
    const rawError =
      typeof failedStep.status === "object" && failedStep.status.type === "failed"
        ? failedStep.status.error
        : activeTask.errorSummary ?? "";
    if (!rawError) return;
    try {
      const report = await diagnoseStepError(
        activeTask.id,
        failedStep.id,
        rawError
      );
      setDiagnostics(report);
    } catch {
      /* silently ignore diagnostic errors */
    }
  }, [activeTask]);

  // Trigger diagnostics automatically when task fails.
  useEffect(() => {
    if (activeTask?.status === "failed" && !diagnostics) {
      runDiagnostics();
    }
  // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [activeTask?.status]);

  return (
    <div className="flex h-full flex-col">
      {/* ── Header ─────────────────────────────────────────────── */}
      <div className="flex items-center justify-between border-b border-[var(--color-border)] px-6 py-4">
        <h2 className="text-lg font-bold">一键部署</h2>
        {isFinished && (
          <button
            onClick={handleNewDeploy}
            className="rounded-md px-3 py-1.5 text-sm text-[var(--color-text-muted)] transition-colors hover:bg-[var(--color-surface-hover)]"
          >
            新建部署
          </button>
        )}
      </div>

      <div className="flex-1 overflow-auto p-6">
        {/* ── Recipe selector (only when no active/running task) ── */}
        {!activeTask && (
          <section className="mb-6">
            <label className="mb-2 block text-sm font-medium text-[var(--color-text-muted)]">
              选择配方
            </label>
            <div className="flex gap-3">
              <select
                value={selectedRecipeId}
                onChange={(e) => setSelectedRecipeId(e.target.value)}
                disabled={recipesLoading}
                className="flex-1 rounded-md border border-[var(--color-border)] bg-[var(--color-surface)] px-3 py-2 text-sm text-[var(--color-text)] focus:outline-none focus:ring-2 focus:ring-[var(--color-primary)]"
              >
                {recipesLoading ? (
                  <option>加载中...</option>
                ) : (
                  recipes.map((r) => (
                    <option key={r.id} value={r.id}>
                      {r.name}
                      {r.author ? ` — ${r.author}` : ""}
                    </option>
                  ))
                )}
              </select>
              <button
                onClick={handleDeploy}
                disabled={!selectedRecipeId || recipesLoading}
                className="rounded-md bg-[var(--color-primary)] px-5 py-2 text-sm font-semibold text-white transition-colors hover:bg-[var(--color-primary-hover)] disabled:opacity-50"
              >
                开始部署
              </button>
            </div>

            {/* Recipe description */}
            {selectedRecipe?.description && (
              <p className="mt-2 text-xs text-[var(--color-text-muted)]">
                {selectedRecipe.description}
              </p>
            )}

            {/* Env requirements */}
            {selectedRecipe && selectedRecipe.envRequirements.length > 0 && (
              <div className="mt-4">
                <p className="mb-2 text-xs font-medium text-[var(--color-text-muted)]">
                  前置要求
                </p>
                <div className="flex flex-wrap gap-2">
                  {selectedRecipe.envRequirements.map((req) => (
                    <span
                      key={req.envId}
                      className="rounded-full border border-[var(--color-border)] px-3 py-1 text-xs text-[var(--color-text-muted)]"
                    >
                      {req.envId}
                      {req.version ? ` ${req.version}` : ""}
                      {req.optional ? " (可选)" : ""}
                    </span>
                  ))}
                </div>
              </div>
            )}
          </section>
        )}

        {/* ── Error ──────────────────────────────────────────────── */}
        {error && (
          <div className="mb-4 rounded-md bg-red-500/10 px-4 py-3 text-sm text-red-400">
            {error}
          </div>
        )}

        {/* ── Active task ─────────────────────────────────────────── */}
        {activeTask && (
          <>
            {/* Task header */}
            <div className="mb-4 flex items-center justify-between">
              <div>
                <h3 className="font-semibold text-[var(--color-text)]">
                  {activeTask.name}
                </h3>
                <p className="mt-0.5 text-xs text-[var(--color-text-muted)]">
                  任务 ID: {activeTask.id.slice(0, 8)}…
                </p>
              </div>
              <TaskStatusBadge status={activeTask.status} />
            </div>

            {/* Overall progress */}
            <div className="mb-2 flex items-center justify-between text-xs text-[var(--color-text-muted)]">
              <span>总进度</span>
              <span>{Math.round(activeTask.progress)}%</span>
            </div>
            <ProgressBar value={activeTask.progress} className="mb-6" />

            {/* Error summary */}
            {activeTask.errorSummary && (
              <div className="mb-4 rounded-md bg-red-500/10 px-4 py-3 text-sm text-red-400">
                {activeTask.errorSummary}
              </div>
            )}

            {/* Diagnostic panel */}
            {diagnostics && activeTask.status === "failed" && (
              <section className="mb-6">
                <div className="mb-2 flex items-center justify-between">
                  <p className="text-sm font-medium text-[var(--color-text-muted)]">
                    🩺 错误诊断
                  </p>
                  <button
                    onClick={() => setDiagnostics(null)}
                    className="text-xs text-[var(--color-text-muted)] hover:text-[var(--color-text)]"
                  >
                    关闭
                  </button>
                </div>
                <ErrorAlert
                  report={diagnostics}
                  onRetry={(_stepId) => {
                    /* future: invoke retry_step command */
                  }}
                />
              </section>
            )}

            {/* Step list */}
            <section className="mb-6">
              <p className="mb-3 text-sm font-medium text-[var(--color-text-muted)]">
                步骤进度
              </p>
              <StepList steps={activeTask.steps} />
            </section>

            {/* Control buttons */}
            {!isFinished && (
              <div className="mb-6 flex gap-3">
                {isRunning && (
                  <button
                    onClick={handlePause}
                    className="rounded-md border border-[var(--color-border)] px-4 py-2 text-sm text-[var(--color-text-muted)] transition-colors hover:bg-[var(--color-surface-hover)]"
                  >
                    ⏸ 暂停
                  </button>
                )}
                {isPaused && (
                  <button
                    onClick={handleResume}
                    className="rounded-md bg-[var(--color-primary)] px-4 py-2 text-sm font-medium text-white transition-colors hover:bg-[var(--color-primary-hover)]"
                  >
                    ▶ 继续
                  </button>
                )}
                <button
                  onClick={handleCancel}
                  className="rounded-md border border-red-500/40 px-4 py-2 text-sm text-red-400 transition-colors hover:bg-red-500/10"
                >
                  ✕ 取消
                </button>
              </div>
            )}

            {/* Log terminal */}
            <section>
              <div className="mb-2 flex items-center justify-between">
                <p className="text-sm font-medium text-[var(--color-text-muted)]">
                  实时日志
                </p>
                <button
                  onClick={() => setShowLog((v) => !v)}
                  className="text-xs text-[var(--color-text-muted)] hover:text-[var(--color-text)]"
                >
                  {showLog ? "折叠" : "展开"}
                </button>
              </div>
              {showLog && (
                <div className="h-64 overflow-hidden rounded-lg border border-[var(--color-border)]">
                  <LogTerminal />
                </div>
              )}
            </section>
          </>
        )}

        {/* ── Empty state ──────────────────────────────────────────── */}
        {!activeTask && !error && recipes.length === 0 && !recipesLoading && (
          <div className="flex flex-col items-center justify-center py-16 text-[var(--color-text-muted)]">
            <p className="text-4xl">📦</p>
            <p className="mt-3 text-sm">暂无可用配方</p>
          </div>
        )}
      </div>
    </div>
  );
}

// ---------------------------------------------------------------------------
// Helper: task status badge
// ---------------------------------------------------------------------------

function TaskStatusBadge({ status }: { status: string }) {
  const cfg: Record<string, { label: string; cls: string }> = {
    idle: { label: "等待", cls: "text-[var(--color-text-muted)]" },
    running: {
      label: "进行中",
      cls: "bg-blue-500/20 text-blue-400",
    },
    paused: { label: "已暂停", cls: "bg-yellow-500/20 text-yellow-400" },
    success: { label: "成功", cls: "bg-green-500/20 text-green-400" },
    failed: { label: "失败", cls: "bg-red-500/20 text-red-400" },
    cancelled: {
      label: "已取消",
      cls: "text-[var(--color-text-muted)]",
    },
  };
  const { label, cls } = cfg[status] ?? { label: status, cls: "" };
  return (
    <span className={`rounded-full px-3 py-1 text-xs font-medium ${cls}`}>
      {label}
    </span>
  );
}
