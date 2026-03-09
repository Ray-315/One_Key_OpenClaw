import type { DiagnosticReport } from "../../ipc/types";
import { FixSuggestionList } from "./FixSuggestionList";

interface ErrorAlertProps {
  report: DiagnosticReport;
  onRetry?: (stepId: string) => void;
  className?: string;
}

/** Shows a diagnostic error report with matched rule and fix suggestions. */
export function ErrorAlert({ report, onRetry, className = "" }: ErrorAlertProps) {
  return (
    <div
      className={`rounded-lg border border-red-500/30 bg-red-500/10 p-4 ${className}`}
    >
      {/* Header */}
      <div className="mb-3 flex items-start gap-2">
        <span className="mt-0.5 text-base text-red-400">✕</span>
        <div className="flex-1 min-w-0">
          <p className="text-sm font-semibold text-red-400">步骤失败</p>
          {report.matchedRule && (
            <p className="mt-0.5 text-xs text-[var(--color-text-muted)]">
              {report.matchedRule.description}
            </p>
          )}
        </div>
        {report.autoFixable && (
          <span className="shrink-0 rounded-full bg-yellow-500/20 px-2 py-0.5 text-xs text-yellow-400">
            可自动修复
          </span>
        )}
      </div>

      {/* Raw error */}
      <pre className="mb-3 max-h-24 overflow-y-auto rounded bg-black/30 px-3 py-2 text-xs text-red-300 whitespace-pre-wrap break-all">
        {report.rawError}
      </pre>

      {/* Fix suggestions */}
      {report.suggestions.length > 0 && (
        <>
          <p className="mb-2 text-xs font-medium text-[var(--color-text-muted)]">
            修复建议
          </p>
          <FixSuggestionList
            suggestions={report.suggestions}
            stepId={report.stepId}
            onRetry={onRetry}
          />
        </>
      )}
    </div>
  );
}
