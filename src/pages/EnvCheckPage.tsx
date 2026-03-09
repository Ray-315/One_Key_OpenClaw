import { useTranslation } from "react-i18next";
import { useEnvProbe } from "../hooks/useEnvProbe";
import { EnvGrid } from "../components/env/EnvGrid";

export function EnvCheckPage() {
  const { t } = useTranslation();
  const { items, loading, error, filter, setFilter, probeAll, probeSingle } =
    useEnvProbe();

  const filterOptions = [
    { value: "all" as const, label: t("env.all") },
    { value: "ok" as const, label: t("env.ok") },
    { value: "missing" as const, label: t("env.missing") },
    { value: "versionMismatch" as const, label: t("env.versionMismatch") },
  ];

  return (
    <div className="flex h-full flex-col">
      {/* Header */}
      <div className="flex items-center justify-between border-b border-[var(--color-border)] px-6 py-4">
        <h2 className="text-lg font-bold">{t("env.title")}</h2>
        <button
          onClick={probeAll}
          disabled={loading}
          className="rounded-md bg-[var(--color-primary)] px-4 py-2 text-sm font-medium text-white transition-colors hover:bg-[var(--color-primary-hover)] disabled:opacity-50"
        >
          {loading ? t("env.probing") : t("env.probeAll")}
        </button>
      </div>

      {/* Filters */}
      <div className="flex items-center gap-2 border-b border-[var(--color-border)] px-6 py-3">
        <span className="text-sm text-[var(--color-text-muted)]">{t("env.filter")}</span>
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
              {t("common.loading")}
            </div>
          </div>
        ) : (
          <EnvGrid items={items} filter={filter} onReprobe={probeSingle} />
        )}
      </div>
    </div>
  );
}
