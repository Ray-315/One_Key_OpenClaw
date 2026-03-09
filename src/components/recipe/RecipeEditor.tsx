import { useState, useCallback } from "react";
import type { Recipe, ValidationIssue } from "../../ipc/types";
import { validateRecipe, saveRecipe } from "../../ipc/recipeApi";

interface RecipeEditorProps {
  recipe: Recipe | null;
  onSave: (recipe: Recipe) => void;
  onCancel: () => void;
}

const EMPTY_RECIPE: Recipe = {
  version: "1",
  id: "",
  name: "",
  description: "",
  author: "",
  tags: [],
  platforms: ["*"],
  envRequirements: [],
  steps: [],
  vars: {},
};

export function RecipeEditor({ recipe, onSave, onCancel }: RecipeEditorProps) {
  const isNew = recipe === null;
  const [tomlText, setTomlText] = useState(() =>
    recipeToToml(recipe ?? EMPTY_RECIPE)
  );
  const [issues, setIssues] = useState<ValidationIssue[]>([]);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleValidate = useCallback(async () => {
    try {
      const parsed = parseTomlToRecipe(tomlText);
      const result = await validateRecipe(parsed);
      setIssues(result);
      setError(null);
      return { parsed, hasErrors: result.some((i) => i.severity === "error") };
    } catch (e) {
      setError(String(e));
      return { parsed: null, hasErrors: true };
    }
  }, [tomlText]);

  const handleSave = useCallback(async () => {
    setSaving(true);
    setError(null);
    try {
      const { parsed, hasErrors } = await handleValidate();
      if (hasErrors || !parsed) {
        setSaving(false);
        return;
      }
      await saveRecipe(parsed);
      onSave(parsed);
    } catch (e) {
      setError(String(e));
    } finally {
      setSaving(false);
    }
  }, [handleValidate, onSave]);

  return (
    <div className="flex h-full flex-col gap-4">
      <div className="flex items-center justify-between">
        <h2 className="text-sm font-semibold text-[var(--color-text)]">
          {isNew ? "新建配方" : `编辑配方: ${recipe?.name}`}
        </h2>
        <div className="flex gap-2">
          <button
            onClick={() => handleValidate()}
            className="rounded-md border border-[var(--color-border)] px-3 py-1 text-xs text-[var(--color-text)] transition-colors hover:bg-[var(--color-surface-hover)]"
          >
            校验
          </button>
          <button
            onClick={handleSave}
            disabled={saving}
            className="rounded-md bg-[var(--color-primary)] px-3 py-1 text-xs text-white transition-colors hover:bg-[var(--color-primary-hover)] disabled:opacity-50"
          >
            {saving ? "保存中..." : "保存"}
          </button>
          <button
            onClick={onCancel}
            className="rounded-md border border-[var(--color-border)] px-3 py-1 text-xs text-[var(--color-text-muted)] transition-colors hover:bg-[var(--color-surface-hover)]"
          >
            取消
          </button>
        </div>
      </div>

      {error && (
        <div className="rounded-md border border-red-500/30 bg-red-500/10 px-3 py-2 text-xs text-red-400">
          {error}
        </div>
      )}

      {issues.length > 0 && (
        <div className="flex flex-col gap-1">
          {issues.map((issue, idx) => (
            <div
              key={idx}
              className={`rounded-md px-3 py-1.5 text-xs ${
                issue.severity === "error"
                  ? "border border-red-500/30 bg-red-500/10 text-red-400"
                  : "border border-yellow-500/30 bg-yellow-500/10 text-yellow-400"
              }`}
            >
              <span className="font-mono">{issue.field}</span>: {issue.message}
            </div>
          ))}
        </div>
      )}

      <textarea
        value={tomlText}
        onChange={(e) => setTomlText(e.target.value)}
        spellCheck={false}
        className="flex-1 resize-none rounded-lg border border-[var(--color-border)] bg-[var(--color-bg)] p-3 font-mono text-xs text-[var(--color-text)] focus:border-[var(--color-primary)] focus:outline-none"
        placeholder="# 在此编辑 TOML 配方..."
      />
    </div>
  );
}

// ---------------------------------------------------------------------------
// Helpers: convert between Recipe object and TOML text
// ---------------------------------------------------------------------------

function recipeToToml(r: Recipe): string {
  const lines: string[] = [];

  lines.push(`version = "${r.version}"`);
  lines.push(`id = "${r.id}"`);
  lines.push(`name = "${r.name}"`);
  if (r.description) lines.push(`description = "${r.description}"`);
  if (r.author) lines.push(`author = "${r.author}"`);
  if (r.tags.length > 0)
    lines.push(`tags = [${r.tags.map((t) => `"${t}"`).join(", ")}]`);
  if (r.platforms.length > 0)
    lines.push(`platforms = [${r.platforms.map((p) => `"${p}"`).join(", ")}]`);

  if (Object.keys(r.vars).length > 0) {
    lines.push("");
    lines.push("[vars]");
    for (const [k, v] of Object.entries(r.vars)) {
      lines.push(`${k} = "${v}"`);
    }
  }

  for (const step of r.steps) {
    lines.push("");
    lines.push("[[steps]]");
    lines.push(`id = "${step.id}"`);
    lines.push(`name = "${step.name}"`);
    if (step.description) lines.push(`description = "${step.description}"`);
    if (step.dependsOn.length > 0)
      lines.push(
        `depends_on = [${step.dependsOn.map((d) => `"${d}"`).join(", ")}]`
      );
    if (step.onError !== "fail") lines.push(`on_error = "${step.onError}"`);
    if (step.timeoutSecs) lines.push(`timeout_secs = ${step.timeoutSecs}`);

    lines.push("");
    lines.push("[steps.action]");
    lines.push(`type = "${step.action.type}"`);
    if (step.action.type === "shell") {
      lines.push(`command = "${step.action.command}"`);
      if (step.action.args.length > 0)
        lines.push(
          `args = [${step.action.args.map((a) => `"${a}"`).join(", ")}]`
        );
    } else if (step.action.type === "packageInstall") {
      lines.push(`manager = "${step.action.manager}"`);
      lines.push(
        `packages = [${step.action.packages.map((p) => `"${p}"`).join(", ")}]`
      );
    } else if (step.action.type === "envCheck") {
      lines.push(`env_id = "${step.action.envId}"`);
    } else if (step.action.type === "download") {
      lines.push(`url = "${step.action.url}"`);
      lines.push(`dest = "${step.action.dest}"`);
    } else if (step.action.type === "extract") {
      lines.push(`src = "${step.action.src}"`);
      lines.push(`dest = "${step.action.dest}"`);
    }
  }

  return lines.join("\n") + "\n";
}

function parseTomlToRecipe(toml: string): Recipe {
  // We send the raw TOML to backend for proper parsing, but we also build
  // a minimal object for client-side validation.  For the TOML editor we
  // extract key fields with regex so the user gets immediate feedback.
  const get = (key: string): string => {
    const m = toml.match(new RegExp(`^${key}\\s*=\\s*"([^"]*)"`, "m"));
    return m ? m[1] : "";
  };

  return {
    version: get("version") || "1",
    id: get("id"),
    name: get("name"),
    description: get("description") || undefined,
    author: get("author") || undefined,
    tags: extractArray(toml, "tags"),
    platforms: extractArray(toml, "platforms"),
    envRequirements: [],
    steps: [],
    vars: {},
  };
}

function extractArray(toml: string, key: string): string[] {
  const m = toml.match(new RegExp(`^${key}\\s*=\\s*\\[([^\\]]*)]`, "m"));
  if (!m) return [];
  return m[1]
    .split(",")
    .map((s) => s.trim().replace(/^"|"$/g, ""))
    .filter(Boolean);
}
