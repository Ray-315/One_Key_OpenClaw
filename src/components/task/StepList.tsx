import type { TaskStep, StepStatus } from "../../ipc/types";

interface StepListProps {
  steps: TaskStep[];
}

function formatDuration(startedAt?: number, finishedAt?: number): string {
  if (!startedAt) return "";
  const end = finishedAt ?? Date.now();
  const secs = Math.round((end - startedAt) / 1000);
  if (secs < 60) return `${secs}s`;
  const m = Math.floor(secs / 60);
  const s = secs % 60;
  return `${m}m ${s}s`;
}

function StatusIcon({ status }: { status: StepStatus }) {
  switch (status.type) {
    case "success":
      return <span className="text-green-400">✅</span>;
    case "running":
      return (
        <span className="inline-block animate-spin text-[var(--color-primary)]">
          🔄
        </span>
      );
    case "failed":
      return <span className="text-red-400">❌</span>;
    case "skipped":
      return <span className="text-yellow-400">⏭️</span>;
    case "cancelled":
      return <span className="text-[var(--color-text-muted)]">🚫</span>;
    case "waiting":
      return <span className="text-blue-400">⏸️</span>;
    default:
      return <span className="text-[var(--color-text-muted)]">⏳</span>;
  }
}

function statusLabel(status: StepStatus): string {
  switch (status.type) {
    case "success":
      return "完成";
    case "running":
      return "进行中";
    case "failed":
      return "失败";
    case "skipped":
      return "已跳过";
    case "cancelled":
      return "已取消";
    case "waiting":
      return "等待中";
    default:
      return "等待";
  }
}

export function StepList({ steps }: StepListProps) {
  if (steps.length === 0) {
    return (
      <p className="text-sm text-[var(--color-text-muted)]">暂无步骤</p>
    );
  }

  return (
    <ol className="flex flex-col gap-2">
      {steps.map((step, idx) => (
        <li
          key={step.id}
          className={`flex items-start gap-3 rounded-lg px-4 py-3 text-sm transition-colors ${
            step.status.type === "running"
              ? "bg-[var(--color-primary)]/10 ring-1 ring-[var(--color-primary)]/30"
              : "bg-[var(--color-surface)]"
          }`}
        >
          {/* Index */}
          <span className="mt-0.5 w-5 shrink-0 text-center text-xs text-[var(--color-text-muted)]">
            {idx + 1}
          </span>

          {/* Icon */}
          <span className="mt-0.5 shrink-0">
            <StatusIcon status={step.status} />
          </span>

          {/* Name + description */}
          <div className="min-w-0 flex-1">
            <p className="font-medium leading-snug text-[var(--color-text)]">
              {step.name}
            </p>
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
              <p className="mt-0.5 text-xs text-yellow-400">
                重试次数: {step.retryCount}/{step.maxRetries}
              </p>
            )}
          </div>

          {/* Status + duration */}
          <div className="shrink-0 text-right">
            <span
              className={`text-xs font-medium ${
                step.status.type === "success"
                  ? "text-green-400"
                  : step.status.type === "failed"
                    ? "text-red-400"
                    : step.status.type === "running"
                      ? "text-[var(--color-primary)]"
                      : "text-[var(--color-text-muted)]"
              }`}
            >
              {statusLabel(step.status)}
            </span>
            {(step.startedAt || step.finishedAt) && (
              <p className="mt-0.5 text-xs text-[var(--color-text-muted)]">
                {formatDuration(step.startedAt, step.finishedAt)}
              </p>
            )}
          </div>
        </li>
      ))}
    </ol>
  );
}
