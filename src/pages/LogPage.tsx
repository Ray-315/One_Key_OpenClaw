import { LogTerminal } from "../components/log/LogTerminal";
import { useLogStream } from "../hooks/useLogStream";
import { useLogStore } from "../store/logStore";

const levelOptions = [
  { value: "all" as const, label: "全部" },
  { value: "debug" as const, label: "Debug" },
  { value: "info" as const, label: "Info" },
  { value: "warn" as const, label: "Warn" },
  { value: "error" as const, label: "Error" },
];

export function LogPage() {
  // Subscribe to all log events
  useLogStream();

  const { filterLevel, setFilterLevel, clear, logs } = useLogStore();

  return (
    <div className="flex h-full flex-col">
      {/* Header */}
      <div className="flex items-center justify-between border-b border-[var(--color-border)] px-6 py-4">
        <h2 className="text-lg font-bold">实时日志</h2>
        <div className="flex items-center gap-3">
          <span className="text-xs text-[var(--color-text-muted)]">
            {logs.length} 条日志
          </span>
          <button
            onClick={clear}
            className="rounded-md px-3 py-1.5 text-sm text-[var(--color-text-muted)] transition-colors hover:bg-[var(--color-surface-hover)] hover:text-[var(--color-text)]"
          >
            清空
          </button>
        </div>
      </div>

      {/* Level filter */}
      <div className="flex items-center gap-2 border-b border-[var(--color-border)] px-6 py-3">
        <span className="text-sm text-[var(--color-text-muted)]">级别:</span>
        {levelOptions.map((opt) => (
          <button
            key={opt.value}
            onClick={() => setFilterLevel(opt.value)}
            className={`rounded-md px-3 py-1 text-sm transition-colors ${
              filterLevel === opt.value
                ? "bg-[var(--color-primary)] text-white"
                : "text-[var(--color-text-muted)] hover:bg-[var(--color-surface-hover)]"
            }`}
          >
            {opt.label}
          </button>
        ))}
      </div>

      {/* Terminal */}
      <div className="flex-1 overflow-hidden p-6">
        <LogTerminal />
      </div>
    </div>
  );
}
