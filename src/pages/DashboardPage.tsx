import { useEffect } from "react";
import { useNavigate } from "react-router-dom";
import { useTranslation } from "react-i18next";
import { useEnvStore } from "../store/envStore";
import { useTaskStore } from "../store/taskStore";
import { useRecipeStore } from "../store/recipeStore";
import type { Task, EnvItem } from "../ipc/types";

// ---------------------------------------------------------------------------
// Helper sub-components
// ---------------------------------------------------------------------------

function StatCard({
  icon,
  label,
  value,
  color,
}: {
  icon: string;
  label: string;
  value: string | number;
  color: string;
}) {
  return (
    <div className="flex items-center gap-4 rounded-xl border border-[var(--color-border)] bg-[var(--color-surface)] p-4">
      <span
        className="flex h-10 w-10 items-center justify-center rounded-lg text-xl"
        style={{ background: `${color}22` }}
      >
        {icon}
      </span>
      <div>
        <p className="text-xs text-[var(--color-text-muted)]">{label}</p>
        <p className="text-xl font-bold text-[var(--color-text)]">{value}</p>
      </div>
    </div>
  );
}

function EnvSummary({ envItems }: { envItems: EnvItem[] }) {
  const { t } = useTranslation();
  const ok = envItems.filter((e) => e.status.type === "ok").length;
  const missing = envItems.filter((e) => e.status.type === "missing").length;
  const mismatch = envItems.filter(
    (e) => e.status.type === "versionMismatch"
  ).length;

  return (
    <div className="rounded-xl border border-[var(--color-border)] bg-[var(--color-surface)] p-4">
      <h3 className="mb-3 text-sm font-semibold text-[var(--color-text)]">
        {t("dashboard.envStatus")}
      </h3>
      {envItems.length === 0 ? (
        <p className="text-xs text-[var(--color-text-muted)]">
          {t("dashboard.envNotChecked")}
        </p>
      ) : (
        <div className="space-y-1">
          {ok > 0 && (
            <div className="flex items-center gap-2">
              <span className="h-2 w-2 rounded-full bg-green-500" />
              <span className="text-xs text-[var(--color-text-muted)]">
                {t("dashboard.itemsOk", { count: ok })}
              </span>
            </div>
          )}
          {mismatch > 0 && (
            <div className="flex items-center gap-2">
              <span className="h-2 w-2 rounded-full bg-yellow-500" />
              <span className="text-xs text-[var(--color-text-muted)]">
                {t("dashboard.itemsMismatch", { count: mismatch })}
              </span>
            </div>
          )}
          {missing > 0 && (
            <div className="flex items-center gap-2">
              <span className="h-2 w-2 rounded-full bg-red-500" />
              <span className="text-xs text-[var(--color-text-muted)]">
                {t("dashboard.itemsMissing", { count: missing })}
              </span>
            </div>
          )}
          <div className="mt-2">
            {envItems.map((item) => (
              <div
                key={item.id}
                className="flex items-center justify-between py-1 border-b border-[var(--color-border)] last:border-0"
              >
                <span className="text-xs text-[var(--color-text)]">
                  {item.name}
                </span>
                <StatusDot status={item.status.type} />
              </div>
            ))}
          </div>
        </div>
      )}
    </div>
  );
}

function StatusDot({ status }: { status: string }) {
  const colorMap: Record<string, string> = {
    ok: "bg-green-500",
    missing: "bg-red-500",
    versionMismatch: "bg-yellow-500",
    error: "bg-red-500",
    checking: "bg-blue-500 animate-pulse",
  };
  return (
    <span
      className={`h-2 w-2 rounded-full ${colorMap[status] ?? "bg-gray-500"}`}
    />
  );
}

function RecentTaskList({ tasks }: { tasks: Task[] }) {
  const { t } = useTranslation();
  const recent = tasks.slice(0, 5);

  const statusKeys: Record<string, { key: string; cls: string }> = {
    idle: { key: "taskStatus.idle", cls: "text-[var(--color-text-muted)]" },
    running: { key: "taskStatus.running", cls: "text-blue-400" },
    paused: { key: "taskStatus.paused", cls: "text-yellow-400" },
    success: { key: "taskStatus.success", cls: "text-green-400" },
    failed: { key: "taskStatus.failed", cls: "text-red-400" },
    cancelled: { key: "taskStatus.cancelled", cls: "text-[var(--color-text-muted)]" },
  };

  return (
    <div className="rounded-xl border border-[var(--color-border)] bg-[var(--color-surface)] p-4">
      <h3 className="mb-3 text-sm font-semibold text-[var(--color-text)]">
        {t("dashboard.recentTasks")}
      </h3>
      {recent.length === 0 ? (
        <p className="text-xs text-[var(--color-text-muted)]">{t("dashboard.noTasks")}</p>
      ) : (
        <ul className="space-y-2">
          {recent.map((task) => {
            const { key, cls } = statusKeys[task.status] ?? {
              key: task.status,
              cls: "",
            };
            return (
              <li
                key={task.id}
                className="flex items-center justify-between gap-2"
              >
                <div className="min-w-0">
                  <p className="truncate text-xs font-medium text-[var(--color-text)]">
                    {task.name}
                  </p>
                  <p className="text-[10px] text-[var(--color-text-muted)]">
                    {new Date(task.createdAt).toLocaleString()}
                  </p>
                </div>
                <span className={`shrink-0 text-xs ${cls}`}>{t(key)}</span>
              </li>
            );
          })}
        </ul>
      )}
    </div>
  );
}

// ---------------------------------------------------------------------------
// Main page
// ---------------------------------------------------------------------------

export function DashboardPage() {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const { items: envItems, probeAll } = useEnvStore();
  const { tasks, loadTasks } = useTaskStore();
  const { recipes, loadRecipes } = useRecipeStore();

  useEffect(() => {
    probeAll();
    loadTasks();
    loadRecipes();
  }, [probeAll, loadTasks, loadRecipes]);

  const taskList = Object.values(tasks).sort(
    (a, b) => b.createdAt - a.createdAt
  );

  const successCount = taskList.filter((t) => t.status === "success").length;
  const failedCount = taskList.filter((t) => t.status === "failed").length;
  const runningCount = taskList.filter((t) => t.status === "running").length;

  const envOkCount = envItems.filter((e) => e.status.type === "ok").length;
  const envTotalCount = envItems.length;

  return (
    <div className="flex h-full flex-col overflow-auto">
      {/* Header */}
      <div className="border-b border-[var(--color-border)] px-6 py-4">
        <h2 className="text-lg font-bold">{t("dashboard.title")}</h2>
        <p className="mt-0.5 text-xs text-[var(--color-text-muted)]">
          {t("dashboard.subtitle")}
        </p>
      </div>

      <div className="flex-1 p-6 space-y-6">
        {/* Stats row */}
        <div className="grid grid-cols-2 gap-4 sm:grid-cols-4">
          <StatCard
            icon="🚀"
            label={t("dashboard.totalRecipes")}
            value={recipes.length}
            color="#6366f1"
          />
          <StatCard
            icon="✅"
            label={t("dashboard.successDeploys")}
            value={successCount}
            color="#22c55e"
          />
          <StatCard
            icon="❌"
            label={t("dashboard.failedCount")}
            value={failedCount}
            color="#ef4444"
          />
          <StatCard
            icon="⚡"
            label={t("dashboard.running")}
            value={runningCount}
            color="#3b82f6"
          />
        </div>

        {/* Environment + Recent tasks */}
        <div className="grid gap-4 sm:grid-cols-2">
          <EnvSummary envItems={envItems} />
          <RecentTaskList tasks={taskList} />
        </div>

        {/* Quick actions */}
        <div className="rounded-xl border border-[var(--color-border)] bg-[var(--color-surface)] p-4">
          <h3 className="mb-3 text-sm font-semibold text-[var(--color-text)]">
            {t("dashboard.quickActions")}
          </h3>
          <div className="flex flex-wrap gap-3">
            <button
              onClick={() => navigate("/deploy")}
              className="rounded-lg bg-[var(--color-primary)] px-4 py-2 text-sm font-medium text-white transition-colors hover:bg-[var(--color-primary-hover)]"
            >
              {t("dashboard.oneKeyDeploy")}
            </button>
            <button
              onClick={() => navigate("/env")}
              className="rounded-lg border border-[var(--color-border)] px-4 py-2 text-sm text-[var(--color-text-muted)] transition-colors hover:bg-[var(--color-surface-hover)] hover:text-[var(--color-text)]"
            >
              {t("dashboard.checkEnv")}
            </button>
            <button
              onClick={() => navigate("/flow")}
              className="rounded-lg border border-[var(--color-border)] px-4 py-2 text-sm text-[var(--color-text-muted)] transition-colors hover:bg-[var(--color-surface-hover)] hover:text-[var(--color-text)]"
            >
              {t("dashboard.taskFlow")}
            </button>
            <button
              onClick={() => navigate("/log")}
              className="rounded-lg border border-[var(--color-border)] px-4 py-2 text-sm text-[var(--color-text-muted)] transition-colors hover:bg-[var(--color-surface-hover)] hover:text-[var(--color-text)]"
            >
              {t("dashboard.viewLog")}
            </button>
          </div>
        </div>

        {/* Env progress bar */}
        {envTotalCount > 0 && (
          <div className="rounded-xl border border-[var(--color-border)] bg-[var(--color-surface)] p-4">
            <div className="mb-2 flex items-center justify-between text-xs text-[var(--color-text-muted)]">
              <span>{t("dashboard.envReadiness")}</span>
              <span>
                {envOkCount}/{envTotalCount}
              </span>
            </div>
            <div className="h-2 overflow-hidden rounded-full bg-[var(--color-border)]">
              <div
                className="h-full rounded-full bg-green-500 transition-all"
                style={{
                  width: `${Math.round((envOkCount / envTotalCount) * 100)}%`,
                }}
              />
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
