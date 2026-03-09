import { useEnvProbe } from "../hooks/useEnvProbe";
import { EnvGrid } from "../components/env/EnvGrid";

const filterOptions = [
  { value: "all" as const, label: "全部" },
  { value: "ok" as const, label: "正常" },
  { value: "missing" as const, label: "缺失" },
  { value: "versionMismatch" as const, label: "版本不符" },
];

export function EnvCheckPage() {
  const { items, loading, error, filter, setFilter, probeAll, probeSingle } =
    useEnvProbe();

  return (
    <div className="flex h-full flex-col">
      {/* Header */}
      <div className="flex items-center justify-between border-b border-[var(--color-border)] px-6 py-4">
        <h2 className="text-lg font-bold">环境检测</h2>
        <button
          onClick={probeAll}
          disabled={loading}
          className="rounded-md bg-[var(--color-primary)] px-4 py-2 text-sm font-medium text-white transition-colors hover:bg-[var(--color-primary-hover)] disabled:opacity-50"
        >
          {loading ? "检测中..." : "全部重新检测"}
        </button>
      </div>

      {/* Filters */}
      <div className="flex items-center gap-2 border-b border-[var(--color-border)] px-6 py-3">
        <span className="text-sm text-[var(--color-text-muted)]">筛选:</span>
        {filterOptions.map((opt) => (
          <button
            key={opt.value}
            onClick={() => setFilter(opt.value)}
            className={`rounded-md px-3 py-1 text-sm transition-colors ${
              filter === opt.value
                ? "bg-[var(--color-primary)] text-white"
                : "text-[var(--color-text-muted)] hover:bg-[var(--color-surface-hover)]"
            }`}
          >
            {opt.label}
          </button>
        ))}
      </div>

      {/* Error display */}
      {error && (
        <div className="mx-6 mt-4 rounded-md bg-red-500/10 px-4 py-3 text-sm text-red-400">
          {error}
        </div>
      )}

      {/* Content */}
      <div className="flex-1 overflow-auto p-6">
        {loading && items.length === 0 ? (
          <div className="flex items-center justify-center py-12">
            <div className="text-[var(--color-text-muted)]">
              <span className="mr-2 inline-block animate-spin">🔄</span>
              正在检测环境...
            </div>
          </div>
        ) : (
          <EnvGrid items={items} filter={filter} onReprobe={probeSingle} />
        )}
      </div>
    </div>
  );
}
