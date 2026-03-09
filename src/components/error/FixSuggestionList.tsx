import type { FixSuggestion } from "../../ipc/types";

interface FixSuggestionListProps {
  suggestions: FixSuggestion[];
  stepId: string;
  onRetry?: (stepId: string) => void;
}

/** Renders a list of fix suggestions, each with an optional action button. */
export function FixSuggestionList({
  suggestions,
  stepId,
  onRetry,
}: FixSuggestionListProps) {
  function handleAction(suggestion: FixSuggestion) {
    if (!suggestion.action) return;
    const { action } = suggestion;
    if (action.type === "retryStep") {
      const targetId = action.stepId === "__current__" ? stepId : action.stepId;
      onRetry?.(targetId);
    } else if (action.type === "openUrl") {
      window.open(action.url, "_blank", "noopener,noreferrer");
    }
    // runCommand and installEnv are handled by the backend via future commands.
  }

  return (
    <ul className="space-y-2">
      {suggestions.map((s, i) => (
        <li
          key={i}
          className="flex items-start gap-3 rounded-md border border-[var(--color-border)] bg-[var(--color-surface)] px-3 py-2"
        >
          <span className="mt-0.5 text-sm text-yellow-400">💡</span>
          <div className="flex-1 min-w-0">
            <p className="text-xs font-medium text-[var(--color-text)]">
              {s.title}
            </p>
            {s.description && (
              <p className="mt-0.5 text-xs text-[var(--color-text-muted)]">
                {s.description}
              </p>
            )}
            {s.action && s.action.type === "runCommand" && (
              <code className="mt-1 block truncate rounded bg-black/30 px-2 py-0.5 text-xs text-green-400">
                {s.action.command} {s.action.args.join(" ")}
              </code>
            )}
          </div>
          {s.action && (
            <button
              onClick={() => handleAction(s)}
              className="shrink-0 rounded-md border border-[var(--color-border)] px-2.5 py-1 text-xs text-[var(--color-text-muted)] transition-colors hover:bg-[var(--color-surface-hover)] hover:text-[var(--color-text)]"
            >
              {s.action.type === "retryStep"
                ? "重试"
                : s.action.type === "openUrl"
                ? "查看"
                : "执行"}
            </button>
          )}
        </li>
      ))}
    </ul>
  );
}
