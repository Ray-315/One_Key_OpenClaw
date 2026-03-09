import type { StepStatus } from "../../ipc/types";

interface StepStatusBadgeProps {
  status: StepStatus;
}

function getConfig(status: StepStatus) {
  switch (status.type) {
    case "pending":
      return { icon: "⏳", label: "等待", cls: "text-[var(--color-text-muted)]" };
    case "waiting":
      return { icon: "🔗", label: "依赖中", cls: "text-[var(--color-text-muted)]" };
    case "running":
      return { icon: "🔄", label: "执行中", cls: "text-blue-400 animate-pulse" };
    case "success":
      return { icon: "✅", label: "成功", cls: "text-[var(--color-success)]" };
    case "failed":
      return { icon: "❌", label: "失败", cls: "text-[var(--color-error)]" };
    case "skipped":
      return { icon: "⏭️", label: "跳过", cls: "text-[var(--color-text-muted)]" };
    case "cancelled":
      return { icon: "🚫", label: "已取消", cls: "text-[var(--color-text-muted)]" };
  }
}

export function StepStatusBadge({ status }: StepStatusBadgeProps) {
  const cfg = getConfig(status);
  return (
    <span className={`inline-flex items-center gap-1 text-xs font-medium ${cfg.cls}`}>
      <span>{cfg.icon}</span>
      <span>{cfg.label}</span>
    </span>
  );
}

interface TaskStep {
  id: string;
  name: string;
  description?: string;
  status: StepStatus;
  startedAt?: number;
  finishedAt?: number;
  exitCode?: number;
  retryCount: number;
  maxRetries: number;
}

interface StepListProps {
  steps: TaskStep[];
}

function formatDuration(startedAt?: number, finishedAt?: number): string {
  if (!startedAt) return "";
  const end = finishedAt ?? Date.now();
  const ms = end - startedAt;
  if (ms < 1000) return `${ms}ms`;
  return `${(ms / 1000).toFixed(1)}s`;
}

export function StepList({ steps }: StepListProps) {
  if (steps.length === 0) {
    return (
      <div className="py-8 text-center text-sm text-[var(--color-text-muted)]">
        暂无步骤
      </div>
    );
  }

  return (
    <ol className="space-y-2">
      {steps.map((step, idx) => (
        <li
          key={step.id}
          className="flex items-start gap-3 rounded-lg border border-[var(--color-border)] bg-[var(--color-surface)] px-4 py-3"
        >
          {/* Step index */}
          <span className="mt-0.5 flex h-6 w-6 shrink-0 items-center justify-center rounded-full bg-[var(--color-surface-hover)] text-xs font-bold text-[var(--color-text-muted)]">
            {idx + 1}
          </span>

          {/* Step info */}
          <div className="min-w-0 flex-1">
            <div className="flex items-center justify-between gap-2">
              <span className="truncate text-sm font-medium text-[var(--color-text)]">
                {step.name}
              </span>
              <div className="flex shrink-0 items-center gap-3">
                {formatDuration(step.startedAt, step.finishedAt) && (
                  <span className="text-xs text-[var(--color-text-muted)]">
                    {formatDuration(step.startedAt, step.finishedAt)}
                  </span>
                )}
                <StepStatusBadge status={step.status} />
              </div>
            </div>

            {step.description && (
              <p className="mt-0.5 text-xs text-[var(--color-text-muted)]">
                {step.description}
              </p>
            )}

            {step.status.type === "failed" && (
              <p className="mt-1 rounded bg-red-500/10 px-2 py-1 text-xs text-red-400">
                {step.status.error}
              </p>
            )}

            {step.retryCount > 0 && (
              <p className="mt-0.5 text-xs text-[var(--color-warning)]">
                重试 {step.retryCount}/{step.maxRetries}
              </p>
            )}
          </div>
        </li>
      ))}
    </ol>
  );
}
