# Architecture Guide / 架构指南

## Overview / 概述

One Key OpenClaw is a cross-platform desktop application built with:

- **Frontend**: React 19 + TypeScript + TailwindCSS 4 + Zustand
- **Backend**: Tauri 2 + Rust + Tokio async runtime
- **IPC**: Tauri's built-in invoke handler for frontend-backend communication

## Backend Architecture / 后端架构

### Module Structure / 模块结构

```
src-tauri/src/
├── lib.rs              # Tauri plugin registration & command handler setup
├── state.rs            # AppState (Mutex-protected shared state)
├── commands/           # IPC command handlers (thin layer)
│   ├── env_commands.rs
│   ├── task_commands.rs
│   ├── recipe_commands.rs
│   ├── error_commands.rs
│   └── plugin_commands.rs
├── env/                # Environment detection
│   ├── prober.rs       # EnvProber: detect tools (node, git, python, etc.)
│   └── platform.rs     # Platform-specific install hints
├── recipe/             # Recipe management
│   ├── schema.rs       # Recipe, RecipeStep, StepAction types + validation
│   ├── parser.rs       # TOML parsing with validation
│   └── registry.rs     # RecipeRegistry: CRUD, file loading, URL fetching
├── task/               # Task execution
│   ├── engine.rs       # Main executor: DAG-based parallel execution
│   ├── graph.rs        # TaskGraph: petgraph DAG for dependency management
│   ├── step.rs         # TaskStep: runtime step status tracking
│   └── state_machine.rs # TaskControl/TaskEvent for pause/resume/cancel
├── error/              # Error diagnostics
│   ├── mod.rs          # AppError enum, ErrorRule, DiagnosticReport types
│   └── engine.rs       # ErrorDiagnosticEngine: regex-based error matching
├── plugin/             # Plugin system
│   ├── api.rs          # PluginManifest, PluginType, PluginInfo types
│   └── manager.rs      # PluginManager: scan, load, unload plugins
└── log/                # Logging infrastructure
```

### Key Design Patterns / 关键设计模式

#### 1. Shared State via AppState
```rust
pub struct AppState {
    pub env_cache: Mutex<Vec<EnvItem>>,
    pub recipes: Mutex<RecipeRegistry>,
    pub tasks: Mutex<HashMap<String, Arc<Mutex<Task>>>>,
    pub task_controls: Mutex<HashMap<String, mpsc::Sender<TaskControl>>>,
    pub plugins: Mutex<PluginManager>,
}
```

All state is protected by `Mutex` for thread safety. Tasks use `Arc<Mutex<Task>>` for concurrent access from the executor.

#### 2. DAG-Based Task Execution

The task engine uses `petgraph` to build a Directed Acyclic Graph from recipe steps. Steps with satisfied dependencies can execute in parallel using Tokio's `JoinSet`.

```
Step A ──┐
         ├──► Step C ──► Step D
Step B ──┘
```

#### 3. Error Diagnostic Pipeline

```
Raw Error Text → Regex Pattern Matching → ErrorRule → FixSuggestions
```

Built-in rules are embedded at compile time from `assets/error_rules.toml`.

#### 4. Plugin Architecture

Plugins are discovered from a `plugins/` directory. Each plugin has a `plugin.toml` manifest declaring its capabilities (recipe_provider, env_probe, step_executor, error_rule, log_sink).

## Frontend Architecture / 前端架构

### State Management / 状态管理

Uses Zustand with separate stores per feature domain:

| Store | Purpose |
|-------|---------|
| `envStore` | Environment detection results |
| `taskStore` | Task execution state |
| `recipeStore` | Recipe management |
| `logStore` | Log messages |

### Routing / 路由

| Path | Page | Description |
|------|------|-------------|
| `/` | DashboardPage | Overview with stats, recent tasks, env status |
| `/env` | EnvCheckPage | Environment detection with filtering |
| `/deploy` | DeployPage | Recipe selection and deployment execution |
| `/recipe` | RecipePage | Recipe management (CRUD, import, marketplace) |
| `/flow` | TaskFlowPage | DAG visualization using React Flow |
| `/log` | LogPage | Terminal-based log viewer (xterm.js) |

### i18n / 国际化

Uses `i18next` + `react-i18next` with:
- Translation files in `src/locales/{zh,en}.json`
- Language preference stored in `localStorage`
- Language switcher in the sidebar

### Component Hierarchy / 组件层次

```
App
├── Sidebar (navigation + language toggle)
└── Routes
    ├── DashboardPage
    │   ├── StatCard
    │   ├── EnvSummary
    │   └── RecentTaskList
    ├── EnvCheckPage → EnvGrid → EnvCard
    ├── DeployPage → StepList + ProgressBar
    ├── RecipePage → RecipeCard + RecipeEditor
    ├── TaskFlowPage → TaskFlow (@xyflow/react)
    └── LogPage → LogTerminal (@xterm/xterm)
```

## Data Flow / 数据流

```
User Action
    │
    ▼
React Component
    │
    ▼
Zustand Store (optimistic UI update)
    │
    ▼
Tauri invoke() → IPC
    │
    ▼
Rust Command Handler
    │
    ▼
Backend Module (env/recipe/task/error/plugin)
    │
    ▼
Response → Zustand Store → React Re-render
```

For real-time updates (task progress, log output), the backend emits Tauri events that the frontend listens to via `listen()`.
