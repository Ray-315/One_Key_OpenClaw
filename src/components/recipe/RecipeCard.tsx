import type { Recipe } from "../../ipc/types";

interface RecipeCardProps {
  recipe: Recipe;
  onEdit: (recipe: Recipe) => void;
  onDelete: (recipeId: string) => void;
  onRun: (recipeId: string) => void;
}

export function RecipeCard({ recipe, onEdit, onDelete, onRun }: RecipeCardProps) {
  const platformLabel = recipe.platforms.length > 0
    ? recipe.platforms.join(" / ")
    : "全平台";

  return (
    <div className="rounded-lg border border-[var(--color-border)] bg-[var(--color-surface)] p-4 transition-colors hover:bg-[var(--color-surface-hover)]">
      <div className="flex items-start justify-between gap-3">
        <div className="min-w-0 flex-1">
          <div className="flex items-center gap-2">
            <span className="text-lg">📦</span>
            <h3 className="truncate text-sm font-semibold text-[var(--color-text)]">
              {recipe.name}
            </h3>
            <span className="shrink-0 rounded bg-blue-500/10 px-1.5 py-0.5 text-xs text-blue-400">
              v{recipe.version}
            </span>
          </div>

          {recipe.description && (
            <p className="mt-1 line-clamp-2 text-xs text-[var(--color-text-muted)]">
              {recipe.description}
            </p>
          )}

          <div className="mt-2 flex flex-wrap items-center gap-2 text-xs text-[var(--color-text-muted)]">
            {recipe.author && (
              <span>
                作者: <span className="text-[var(--color-text)]">{recipe.author}</span>
              </span>
            )}
            <span className="text-[var(--color-border)]">|</span>
            <span>{platformLabel}</span>
          </div>

          {recipe.tags.length > 0 && (
            <div className="mt-2 flex flex-wrap gap-1">
              {recipe.tags.map((tag) => (
                <span
                  key={tag}
                  className="rounded-full bg-[var(--color-surface-hover)] px-2 py-0.5 text-xs text-[var(--color-text-muted)]"
                >
                  {tag}
                </span>
              ))}
            </div>
          )}
        </div>

        <div className="flex shrink-0 items-center gap-1">
          <span className="text-xs text-[var(--color-text-muted)]">
            {recipe.steps.length} 步骤
          </span>
        </div>
      </div>

      <div className="mt-3 flex items-center gap-2 border-t border-[var(--color-border)] pt-3">
        <button
          onClick={() => onRun(recipe.id)}
          className="rounded-md bg-[var(--color-primary)] px-3 py-1 text-xs text-white transition-colors hover:bg-[var(--color-primary-hover)]"
        >
          运行
        </button>
        <button
          onClick={() => onEdit(recipe)}
          className="rounded-md border border-[var(--color-border)] px-3 py-1 text-xs text-[var(--color-text)] transition-colors hover:bg-[var(--color-surface-hover)]"
        >
          编辑
        </button>
        <div className="flex-1" />
        <button
          onClick={() => onDelete(recipe.id)}
          className="rounded-md px-3 py-1 text-xs text-red-400 transition-colors hover:bg-red-500/10"
        >
          删除
        </button>
      </div>
    </div>
  );
}
