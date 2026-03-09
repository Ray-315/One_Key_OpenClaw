import { useEffect, useState, useCallback } from "react";
import { useNavigate } from "react-router-dom";
import { useRecipeStore } from "../store/recipeStore";
import { RecipeCard } from "../components/recipe/RecipeCard";
import { RecipeEditor } from "../components/recipe/RecipeEditor";
import {
  deleteRecipe,
  loadRecipeFile,
  fetchRecipeUrl,
} from "../ipc/recipeApi";
import type { Recipe } from "../ipc/types";

// ---------------------------------------------------------------------------
// Community marketplace sample data (prototype)
// ---------------------------------------------------------------------------

interface MarketplaceRecipe {
  id: string;
  name: string;
  description: string;
  author: string;
  version: string;
  tags: string[];
  url: string;
  downloads: number;
}

const MARKETPLACE_RECIPES: MarketplaceRecipe[] = [
  {
    id: "community-docker-compose",
    name: "Docker Compose 一键部署",
    description: "基于 Docker Compose 的多容器应用快速部署配方",
    author: "community",
    version: "1.0.0",
    tags: ["docker", "compose", "deploy"],
    url: "https://raw.githubusercontent.com/example/recipes/main/docker-compose.toml",
    downloads: 1280,
  },
  {
    id: "community-k8s-basic",
    name: "Kubernetes 基础部署",
    description: "Kubernetes 集群基础环境搭建与应用部署",
    author: "k8s-contrib",
    version: "0.9.0",
    tags: ["kubernetes", "k8s", "cloud"],
    url: "https://raw.githubusercontent.com/example/recipes/main/k8s-basic.toml",
    downloads: 856,
  },
  {
    id: "community-python-ml",
    name: "Python ML 环境搭建",
    description: "配置 Python 机器学习开发环境（PyTorch / TensorFlow / Jupyter）",
    author: "ml-team",
    version: "1.2.0",
    tags: ["python", "ml", "ai"],
    url: "https://raw.githubusercontent.com/example/recipes/main/python-ml.toml",
    downloads: 2340,
  },
  {
    id: "community-rust-dev",
    name: "Rust 开发环境",
    description: "Rust 工具链 + 常用开发工具一键安装",
    author: "rustacean",
    version: "1.1.0",
    tags: ["rust", "dev", "toolchain"],
    url: "https://raw.githubusercontent.com/example/recipes/main/rust-dev.toml",
    downloads: 1670,
  },
];

// ---------------------------------------------------------------------------
// RecipePage
// ---------------------------------------------------------------------------

type ViewMode = "list" | "editor" | "marketplace";

export function RecipePage() {
  const { recipes, loading, error, loadRecipes } = useRecipeStore();
  const navigate = useNavigate();

  const [viewMode, setViewMode] = useState<ViewMode>("list");
  const [editingRecipe, setEditingRecipe] = useState<Recipe | null>(null);
  const [importError, setImportError] = useState<string | null>(null);
  const [urlInput, setUrlInput] = useState("");
  const [showUrlImport, setShowUrlImport] = useState(false);

  useEffect(() => {
    loadRecipes();
  }, [loadRecipes]);

  // ---- handlers ----

  const handleEdit = useCallback((recipe: Recipe) => {
    setEditingRecipe(recipe);
    setViewMode("editor");
  }, []);

  const handleNew = useCallback(() => {
    setEditingRecipe(null);
    setViewMode("editor");
  }, []);

  const handleDelete = useCallback(
    async (recipeId: string) => {
      try {
        await deleteRecipe(recipeId);
        useRecipeStore.getState().removeRecipe(recipeId);
      } catch (e) {
        setImportError(String(e));
      }
    },
    []
  );

  const handleRun = useCallback(
    (recipeId: string) => {
      navigate(`/deploy?recipe=${recipeId}`);
    },
    [navigate]
  );

  const handleEditorSave = useCallback(
    (recipe: Recipe) => {
      useRecipeStore.getState().upsertRecipe(recipe);
      setViewMode("list");
      setEditingRecipe(null);
    },
    []
  );

  const handleEditorCancel = useCallback(() => {
    setViewMode("list");
    setEditingRecipe(null);
  }, []);

  const handleImportFile = useCallback(async () => {
    setImportError(null);
    try {
      // Use a simple prompt for file path (Tauri file dialog could be used
      // with tauri-plugin-dialog in a future enhancement).
      const path = prompt("请输入配方文件路径 (.toml):");
      if (!path) return;
      const recipe = await loadRecipeFile(path);
      useRecipeStore.getState().upsertRecipe(recipe);
    } catch (e) {
      setImportError(String(e));
    }
  }, []);

  const handleImportUrl = useCallback(async () => {
    if (!urlInput.trim()) return;
    setImportError(null);
    try {
      const recipe = await fetchRecipeUrl(urlInput.trim());
      useRecipeStore.getState().upsertRecipe(recipe);
      setUrlInput("");
      setShowUrlImport(false);
    } catch (e) {
      setImportError(String(e));
    }
  }, [urlInput]);

  const handleInstallMarketplace = useCallback(
    async (item: MarketplaceRecipe) => {
      setImportError(null);
      try {
        const recipe = await fetchRecipeUrl(item.url);
        useRecipeStore.getState().upsertRecipe(recipe);
      } catch (e) {
        setImportError(
          `无法安装 "${item.name}": ${String(e)}`
        );
      }
    },
    []
  );

  // ---- render ----

  if (viewMode === "editor") {
    return (
      <div className="flex h-full flex-col overflow-auto p-6">
        <RecipeEditor
          recipe={editingRecipe}
          onSave={handleEditorSave}
          onCancel={handleEditorCancel}
        />
      </div>
    );
  }

  return (
    <div className="flex h-full flex-col overflow-auto p-6">
      {/* Header */}
      <div className="mb-4 flex items-center justify-between">
        <div className="flex items-center gap-3">
          <h1 className="text-lg font-bold text-[var(--color-text)]">
            📦 配方库
          </h1>
          <div className="flex gap-1 rounded-md border border-[var(--color-border)] p-0.5">
            <button
              onClick={() => setViewMode("list")}
              className={`rounded px-2.5 py-1 text-xs transition-colors ${
                viewMode === "list"
                  ? "bg-[var(--color-primary)] text-white"
                  : "text-[var(--color-text-muted)] hover:text-[var(--color-text)]"
              }`}
            >
              我的配方
            </button>
            <button
              onClick={() => setViewMode("marketplace")}
              className={`rounded px-2.5 py-1 text-xs transition-colors ${
                viewMode === "marketplace"
                  ? "bg-[var(--color-primary)] text-white"
                  : "text-[var(--color-text-muted)] hover:text-[var(--color-text)]"
              }`}
            >
              配方市场
            </button>
          </div>
        </div>

        <div className="flex gap-2">
          <button
            onClick={handleImportFile}
            className="rounded-md border border-[var(--color-border)] px-3 py-1.5 text-xs text-[var(--color-text)] transition-colors hover:bg-[var(--color-surface-hover)]"
          >
            导入文件
          </button>
          <button
            onClick={() => setShowUrlImport(!showUrlImport)}
            className="rounded-md border border-[var(--color-border)] px-3 py-1.5 text-xs text-[var(--color-text)] transition-colors hover:bg-[var(--color-surface-hover)]"
          >
            从 URL 导入
          </button>
          <button
            onClick={handleNew}
            className="rounded-md bg-[var(--color-primary)] px-3 py-1.5 text-xs text-white transition-colors hover:bg-[var(--color-primary-hover)]"
          >
            新建配方
          </button>
        </div>
      </div>

      {/* URL import bar */}
      {showUrlImport && (
        <div className="mb-4 flex items-center gap-2 rounded-lg border border-[var(--color-border)] bg-[var(--color-surface)] p-3">
          <input
            type="text"
            value={urlInput}
            onChange={(e) => setUrlInput(e.target.value)}
            placeholder="https://example.com/recipe.toml"
            className="flex-1 rounded-md border border-[var(--color-border)] bg-[var(--color-bg)] px-3 py-1.5 text-xs text-[var(--color-text)] focus:border-[var(--color-primary)] focus:outline-none"
          />
          <button
            onClick={handleImportUrl}
            className="rounded-md bg-[var(--color-primary)] px-3 py-1.5 text-xs text-white transition-colors hover:bg-[var(--color-primary-hover)]"
          >
            导入
          </button>
          <button
            onClick={() => setShowUrlImport(false)}
            className="px-2 text-xs text-[var(--color-text-muted)] hover:text-[var(--color-text)]"
          >
            ✕
          </button>
        </div>
      )}

      {/* Error */}
      {(error || importError) && (
        <div className="mb-4 rounded-md border border-red-500/30 bg-red-500/10 px-3 py-2 text-xs text-red-400">
          {error || importError}
        </div>
      )}

      {/* My recipes */}
      {viewMode === "list" && (
        <>
          {loading && (
            <p className="py-8 text-center text-xs text-[var(--color-text-muted)]">
              加载中...
            </p>
          )}

          {!loading && recipes.length === 0 && (
            <div className="flex flex-col items-center justify-center gap-3 py-16 text-center">
              <span className="text-3xl">📭</span>
              <p className="text-sm text-[var(--color-text-muted)]">
                暂无配方。点击"新建配方"或从文件 / URL 导入。
              </p>
            </div>
          )}

          <div className="grid gap-3">
            {recipes.map((recipe) => (
              <RecipeCard
                key={recipe.id}
                recipe={recipe}
                onEdit={handleEdit}
                onDelete={handleDelete}
                onRun={handleRun}
              />
            ))}
          </div>
        </>
      )}

      {/* Marketplace (prototype) */}
      {viewMode === "marketplace" && (
        <div className="flex flex-col gap-4">
          <p className="text-xs text-[var(--color-text-muted)]">
            浏览社区共享的配方。点击"安装"将配方添加到你的本地库中。
          </p>

          <div className="grid gap-3">
            {MARKETPLACE_RECIPES.map((item) => (
              <MarketplaceCard
                key={item.id}
                item={item}
                installed={recipes.some((r) => r.id === item.id)}
                onInstall={() => handleInstallMarketplace(item)}
              />
            ))}
          </div>
        </div>
      )}
    </div>
  );
}

// ---------------------------------------------------------------------------
// Marketplace card sub-component
// ---------------------------------------------------------------------------

function MarketplaceCard({
  item,
  installed,
  onInstall,
}: {
  item: MarketplaceRecipe;
  installed: boolean;
  onInstall: () => void;
}) {
  return (
    <div className="rounded-lg border border-[var(--color-border)] bg-[var(--color-surface)] p-4 transition-colors hover:bg-[var(--color-surface-hover)]">
      <div className="flex items-start justify-between gap-3">
        <div className="min-w-0 flex-1">
          <div className="flex items-center gap-2">
            <span className="text-lg">🌐</span>
            <h3 className="truncate text-sm font-semibold text-[var(--color-text)]">
              {item.name}
            </h3>
            <span className="shrink-0 rounded bg-blue-500/10 px-1.5 py-0.5 text-xs text-blue-400">
              v{item.version}
            </span>
          </div>
          <p className="mt-1 text-xs text-[var(--color-text-muted)]">
            {item.description}
          </p>
          <div className="mt-2 flex items-center gap-3 text-xs text-[var(--color-text-muted)]">
            <span>作者: {item.author}</span>
            <span>⬇ {item.downloads.toLocaleString()}</span>
          </div>
          {item.tags.length > 0 && (
            <div className="mt-2 flex flex-wrap gap-1">
              {item.tags.map((tag) => (
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

        <button
          onClick={onInstall}
          disabled={installed}
          className={`shrink-0 rounded-md px-3 py-1.5 text-xs transition-colors ${
            installed
              ? "cursor-default border border-green-500/30 text-green-400"
              : "bg-[var(--color-primary)] text-white hover:bg-[var(--color-primary-hover)]"
          }`}
        >
          {installed ? "已安装" : "安装"}
        </button>
      </div>
    </div>
  );
}
