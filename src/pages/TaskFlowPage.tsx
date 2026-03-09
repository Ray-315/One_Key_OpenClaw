import { useEffect, useState } from "react";
import { useRecipeStore } from "../store/recipeStore";
import { useTaskStore } from "../store/taskStore";
import { getRecipeGraph } from "../ipc/errorApi";
import { TaskFlow } from "../components/task/TaskFlow";
import type { TaskGraphData, StepStatus } from "../ipc/types";

export function TaskFlowPage() {
  const { recipes, loadRecipes } = useRecipeStore();
  const { tasks, activeTaskId } = useTaskStore();

  const [selectedRecipeId, setSelectedRecipeId] = useState<string>("");
  const [graphData, setGraphData] = useState<TaskGraphData | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    loadRecipes();
  }, [loadRecipes]);

  // Auto-select first recipe.
  useEffect(() => {
    if (!selectedRecipeId && recipes.length > 0) {
      setSelectedRecipeId(recipes[0].id);
    }
  }, [recipes, selectedRecipeId]);

  // Load graph whenever selected recipe changes.
  useEffect(() => {
    if (!selectedRecipeId) return;
    setLoading(true);
    setError(null);
    getRecipeGraph(selectedRecipeId)
      .then(setGraphData)
      .catch((e: unknown) => setError(String(e)))
      .finally(() => setLoading(false));
  }, [selectedRecipeId]);

  // Compute step statuses from the active task.
  const stepStatuses: Record<string, StepStatus> = {};
  const activeTask = activeTaskId ? tasks[activeTaskId] : null;
  if (activeTask) {
    for (const step of activeTask.steps) {
      stepStatuses[step.id] = step.status;
    }
  }

  return (
    <div className="flex h-full flex-col">
      {/* Header */}
      <div className="flex items-center gap-4 border-b border-[var(--color-border)] px-6 py-4">
        <h2 className="text-lg font-bold">任务编排可视化</h2>
        <select
          value={selectedRecipeId}
          onChange={(e) => setSelectedRecipeId(e.target.value)}
          className="rounded-md border border-[var(--color-border)] bg-[var(--color-surface)] px-3 py-1.5 text-sm text-[var(--color-text)] focus:outline-none focus:ring-2 focus:ring-[var(--color-primary)]"
        >
          {recipes.map((r) => (
            <option key={r.id} value={r.id}>
              {r.name}
            </option>
          ))}
        </select>
        {activeTask && (
          <div className="flex items-center gap-2 rounded-full border border-[var(--color-border)] px-3 py-1 text-xs text-[var(--color-text-muted)]">
            <span className="h-1.5 w-1.5 rounded-full bg-blue-400 animate-pulse" />
            运行中: {activeTask.name.slice(0, 20)}
            {activeTask.name.length > 20 ? "…" : ""}
          </div>
        )}
      </div>

      {/* Legend */}
      <div className="flex items-center gap-4 border-b border-[var(--color-border)] bg-[var(--color-surface)] px-6 py-2">
        <span className="text-xs text-[var(--color-text-muted)]">图例：</span>
        {[
          { color: "#6b7280", label: "等待" },
          { color: "#3b82f6", label: "进行中" },
          { color: "#22c55e", label: "成功" },
          { color: "#ef4444", label: "失败" },
          { color: "#f59e0b", label: "已跳过" },
          { color: "#6b7280", label: "已取消" },
        ].map(({ color, label }) => (
          <div key={label} className="flex items-center gap-1.5">
            <span
              className="h-3 w-3 rounded"
              style={{ background: `${color}44`, border: `1.5px solid ${color}99` }}
            />
            <span className="text-xs text-[var(--color-text-muted)]">{label}</span>
          </div>
        ))}
      </div>

      {/* Main canvas */}
      <div className="relative flex-1 overflow-hidden">
        {loading && (
          <div className="absolute inset-0 z-10 flex items-center justify-center bg-[var(--color-bg)]/80">
            <p className="text-sm text-[var(--color-text-muted)]">加载中…</p>
          </div>
        )}
        {error && (
          <div className="absolute inset-0 z-10 flex items-center justify-center">
            <div className="rounded-lg border border-red-500/30 bg-red-500/10 px-6 py-4 text-sm text-red-400">
              {error}
            </div>
          </div>
        )}
        {graphData && !loading && (
          <TaskFlow
            graphData={graphData}
            stepStatuses={stepStatuses}
            className="h-full w-full"
          />
        )}
        {!graphData && !loading && !error && (
          <div className="flex h-full items-center justify-center text-[var(--color-text-muted)]">
            <p className="text-sm">选择一个配方以查看 DAG 图</p>
          </div>
        )}
      </div>
    </div>
  );
}
