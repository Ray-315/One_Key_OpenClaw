import type { EnvItem } from "../../ipc/types";
import { StatusBadge } from "../common/StatusBadge";
import { getPlatform } from "../../utils/platform";

interface EnvCardProps {
  item: EnvItem;
  onReprobe: (id: string) => void;
}

export function EnvCard({ item, onReprobe }: EnvCardProps) {
  const platform = getPlatform();
  const hint =
    item.installHint?.[platform === "macos" ? "macos" : platform === "windows" ? "windows" : "linux"];

  return (
    <div className="rounded-lg border border-[var(--color-border)] bg-[var(--color-surface)] p-4 transition-colors hover:bg-[var(--color-surface-hover)]">
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-3">
          <span className="text-2xl">{getIcon(item.id)}</span>
          <div>
            <h3 className="font-semibold text-[var(--color-text)]">
              {item.name}
            </h3>
            <p className="text-xs text-[var(--color-text-muted)]">
              {item.path || "未检测到路径"}
            </p>
          </div>
        </div>
        <StatusBadge status={item.status} />
      </div>

      <div className="mt-3 flex items-center justify-between">
        <div className="text-sm text-[var(--color-text-muted)]">
          {item.version && (
            <span>
              版本: <span className="text-[var(--color-text)]">v{item.version}</span>
            </span>
          )}
          {item.requiredVersion && (
            <span className="ml-3">
              要求: {item.requiredVersion}
            </span>
          )}
        </div>
        <button
          onClick={() => onReprobe(item.id)}
          className="rounded px-2 py-1 text-xs text-[var(--color-primary)] transition-colors hover:bg-[var(--color-primary)]/10"
        >
          重新检测
        </button>
      </div>

      {item.status.type === "missing" && hint && (
        <div className="mt-2 rounded bg-[var(--color-bg)] px-3 py-2 text-xs text-[var(--color-text-muted)]">
          💡 建议: <code className="text-[var(--color-warning)]">{hint}</code>
        </div>
      )}

      {item.status.type === "versionMismatch" && (
        <div className="mt-2 rounded bg-[var(--color-bg)] px-3 py-2 text-xs text-[var(--color-text-muted)]">
          ⚠️ 当前版本 {item.status.found}，要求 {item.status.required}
        </div>
      )}

      {item.status.type === "error" && (
        <div className="mt-2 rounded bg-red-500/5 px-3 py-2 text-xs text-red-400">
          ⚠️ {item.status.message}
        </div>
      )}
    </div>
  );
}

function getIcon(id: string): string {
  switch (id) {
    case "node":
      return "🟢";
    case "git":
      return "🔀";
    case "python":
      return "🐍";
    case "rustc":
      return "🦀";
    case "docker":
      return "🐳";
    default:
      return "📦";
  }
}
