# One Key OpenClaw — 完整系统架构方案

> 基于 Tauri 2 + Rust + React + TypeScript + TailwindCSS 的可视化一键部署桌面工具

---

## 目录

1. [项目目标与功能概述](#1-项目目标与功能概述)
2. [技术选型](#2-技术选型)
3. [系统架构总览](#3-系统架构总览)
4. [模块划分](#4-模块划分)
5. [数据结构设计](#5-数据结构设计)
6. [Rust 后端接口设计（Tauri Commands）](#6-rust-后端接口设计tauri-commands)
7. [前端页面信息架构](#7-前端页面信息架构)
8. [任务状态机](#8-任务状态机)
9. [错误处理机制](#9-错误处理机制)
10. [配方系统设计（Recipe System）](#10-配方系统设计recipe-system)
11. [插件式扩展架构](#11-插件式扩展架构)
12. [多平台适配方案](#12-多平台适配方案)
13. [实时日志流架构](#13-实时日志流架构)
14. [目录结构](#14-目录结构)
15. [开发路线图](#15-开发路线图)

---

## 1. 项目目标与功能概述

**One Key OpenClaw** 是一款跨平台桌面工具，为 OpenClaw、Claude Code 及相关本地 Agent 工具链提供：

| 功能 | 描述 |
|------|------|
| 可视化环境检测 | 自动检测 OS、Node.js、Python、Rust、Git、Docker 等运行时环境 |
| 依赖安装 | 自动安装缺失依赖，支持多包管理器（npm/pip/cargo/brew/winget/apt） |
| 一键部署 | 通过声明式配方文件完成全流程部署 |
| 错误诊断与自动修复 | 捕获命令错误，提供上下文诊断，支持自动重试与修复建议 |
| 任务编排 | DAG 图任务调度，支持串行/并行/条件分支 |
| 实时日志流 | 通过 Tauri 事件系统将子进程日志推送到前端 |
| 分步骤安装状态 | 每个安装步骤独立跟踪状态，支持断点续装 |
| 插件扩展 | 第三方配方与诊断插件热加载 |

---

## 2. 技术选型

### 桌面框架
- **Tauri 2**：轻量级 Rust 驱动桌面框架，WebView 前端 + Rust 后端
  - 使用 `tauri::command` 暴露 IPC 接口
  - 使用 `tauri::Emitter` / `tauri::Listener` 实现事件双向通信
  - 使用 `tauri-plugin-shell` 执行系统命令
  - 使用 `tauri-plugin-fs` 进行文件操作
  - 使用 `tauri-plugin-store` 持久化配置

### 核心执行引擎（Rust）
- **tokio**：异步运行时，任务并发调度
- **serde / serde_json**：数据序列化
- **anyhow / thiserror**：结构化错误处理
- **which**：跨平台命令可用性检测
- **semver**：版本约束解析
- **petgraph**：DAG 任务依赖图
- **notify**：文件系统监听（配方热重载）

### 前端
- **React 18 + TypeScript**：UI 框架
- **TailwindCSS**：原子化样式
- **Zustand**：轻量状态管理
- **React Router v6**：单页路由
- **Framer Motion**：动画与过渡
- **Xterm.js**：终端日志组件
- **React Flow**：任务 DAG 可视化
- **Radix UI**：无障碍原语组件

---

## 3. 系统架构总览

```
┌─────────────────────────────────────────────────────────────────┐
│                        前端（WebView）                           │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌───────────────┐   │
│  │ 环境检测  │  │ 任务编排  │  │ 实时日志  │  │ 配方管理/插件 │   │
│  │   页面   │  │   页面   │  │   终端   │  │     页面     │   │
│  └────┬─────┘  └────┬─────┘  └────┬─────┘  └──────┬────────┘   │
│       │              │              │                │            │
│  ┌────▼──────────────▼──────────────▼────────────────▼────────┐  │
│  │                  Zustand 状态层 + IPC 适配层                 │  │
│  └─────────────────────────────┬───────────────────────────────┘  │
└────────────────────────────────│────────────────────────────────┘
                                 │ Tauri IPC (invoke / emit)
┌────────────────────────────────▼────────────────────────────────┐
│                        Rust 后端（Tauri Core）                   │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────────┐  │
│  │ 环境探测器   │  │ 任务执行引擎  │  │     配方解析器           │  │
│  │ EnvProber   │  │TaskExecutor │  │   RecipeParser          │  │
│  └─────────────┘  └─────────────┘  └─────────────────────────┘  │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────────┐  │
│  │ 错误诊断引擎  │  │ 插件管理器   │  │     日志管道             │  │
│  │ErrorEngine  │  │PluginMgr    │  │   LogPipeline           │  │
│  └─────────────┘  └─────────────┘  └─────────────────────────┘  │
│                                                                  │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │              平台抽象层 (Platform HAL)                    │   │
│  │   macOS / Windows / Linux 适配                           │   │
│  └──────────────────────────────────────────────────────────┘   │
└──────────────────────────────────────────────────────────────────┘
```

### 通信模型

| 方向 | 机制 | 用途 |
|------|------|------|
| 前端 → 后端 | `invoke(command, args)` | 触发操作、查询状态 |
| 后端 → 前端 | `emit(event, payload)` | 日志流、任务进度推送 |
| 后端内部 | `tokio::mpsc` 通道 | 模块间异步通信 |

---

## 4. 模块划分

### 4.1 Rust 后端模块

```
src-tauri/src/
├── main.rs                  # Tauri App 入口，注册 commands 和 plugins
├── lib.rs                   # 公共类型导出
├── env/
│   ├── mod.rs               # 环境探测模块入口
│   ├── prober.rs            # EnvProber：检测各环境项
│   └── platform.rs          # 平台相关的命令路径、包管理器
├── recipe/
│   ├── mod.rs               # 配方模块入口
│   ├── parser.rs            # TOML/YAML 配方解析
│   ├── schema.rs            # 配方数据结构定义
│   └── registry.rs          # 本地/远程配方注册表
├── task/
│   ├── mod.rs               # 任务模块入口
│   ├── engine.rs            # TaskExecutor：DAG 调度执行
│   ├── state_machine.rs     # TaskStateMachine：状态转换
│   ├── step.rs              # 单步任务执行
│   └── graph.rs             # DAG 图构建 (petgraph)
├── error/
│   ├── mod.rs               # 错误类型定义
│   ├── engine.rs            # ErrorDiagnosticEngine
│   └── rules.rs             # 错误匹配规则表
├── log/
│   ├── mod.rs               # 日志模块
│   └── pipeline.rs          # LogPipeline：子进程输出 → Tauri emit
├── plugin/
│   ├── mod.rs               # 插件模块入口
│   ├── manager.rs           # PluginManager：热加载
│   └── api.rs               # 插件 API 接口规范
└── commands/
    ├── mod.rs               # 所有 Tauri commands 汇总注册
    ├── env_commands.rs      # 环境检测命令
    ├── task_commands.rs     # 任务控制命令
    ├── recipe_commands.rs   # 配方管理命令
    └── plugin_commands.rs   # 插件管理命令
```

### 4.2 前端模块

```
src/
├── main.tsx                 # React 入口
├── App.tsx                  # 路由布局
├── pages/
│   ├── DashboardPage.tsx    # 仪表板：总览
│   ├── EnvCheckPage.tsx     # 环境检测页
│   ├── DeployPage.tsx       # 一键部署页
│   ├── TaskFlowPage.tsx     # 任务编排可视化页
│   ├── LogPage.tsx          # 日志终端页
│   ├── RecipePage.tsx       # 配方管理页
│   └── SettingsPage.tsx     # 设置页
├── components/
│   ├── env/
│   │   ├── EnvCard.tsx      # 单个环境项卡片
│   │   └── EnvGrid.tsx      # 环境检测网格
│   ├── task/
│   │   ├── TaskNode.tsx     # React Flow 任务节点
│   │   ├── TaskFlow.tsx     # DAG 可视化画布
│   │   ├── StepList.tsx     # 步骤列表（带状态）
│   │   └── ProgressBar.tsx  # 任务进度条
│   ├── log/
│   │   └── LogTerminal.tsx  # Xterm.js 终端组件
│   ├── recipe/
│   │   ├── RecipeCard.tsx   # 配方卡片
│   │   └── RecipeEditor.tsx # 配方 TOML 编辑器
│   └── common/
│       ├── StatusBadge.tsx  # 状态徽章
│       ├── ErrorAlert.tsx   # 错误提示组件
│       └── Sidebar.tsx      # 侧边导航
├── store/
│   ├── envStore.ts          # 环境状态
│   ├── taskStore.ts         # 任务状态
│   ├── recipeStore.ts       # 配方状态
│   └── logStore.ts          # 日志缓冲
├── hooks/
│   ├── useEnvProbe.ts       # 调用环境检测
│   ├── useTaskRunner.ts     # 任务执行控制
│   ├── useLogStream.ts      # 订阅日志事件
│   └── useTauriEvent.ts     # 通用事件订阅
├── ipc/
│   ├── envApi.ts            # 封装环境检测 invoke
│   ├── taskApi.ts           # 封装任务控制 invoke
│   ├── recipeApi.ts         # 封装配方管理 invoke
│   └── types.ts             # 与 Rust 共享的 TS 类型
└── utils/
    ├── platform.ts          # 平台判断工具
    └── format.ts            # 时间/大小格式化
```

---

## 5. 数据结构设计

### 5.1 环境项 `EnvItem`

```rust
// src-tauri/src/env/prober.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EnvItem {
    /// 环境项唯一标识，如 "node", "python", "git"
    pub id: String,
    /// 显示名称
    pub name: String,
    /// 检测状态
    pub status: EnvStatus,
    /// 检测到的版本（若可用）
    pub version: Option<String>,
    /// 要求的最低版本约束（semver）
    pub required_version: Option<String>,
    /// 可执行文件路径
    pub path: Option<PathBuf>,
    /// 推荐的安装命令（按平台）
    pub install_hint: Option<InstallHint>,
    /// 上次检测时间戳（Unix ms）
    pub checked_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum EnvStatus {
    Ok,
    Missing,
    VersionMismatch { found: String, required: String },
    Error { message: String },
    Checking,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallHint {
    pub macos: Option<String>,
    pub windows: Option<String>,
    pub linux: Option<String>,
}
```

### 5.2 任务与步骤

```rust
// src-tauri/src/task/step.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskStep {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    /// 执行类型
    pub action: StepAction,
    /// 该步骤依赖的步骤 ID 列表
    pub depends_on: Vec<String>,
    pub status: StepStatus,
    pub started_at: Option<u64>,
    pub finished_at: Option<u64>,
    pub exit_code: Option<i32>,
    /// 失败时重试次数
    pub retry_count: u8,
    pub max_retries: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum StepAction {
    /// 执行 shell 命令
    Shell { command: String, args: Vec<String>, env: HashMap<String, String> },
    /// 安装包管理器包
    PackageInstall { manager: PackageManager, packages: Vec<String> },
    /// 检查环境项
    EnvCheck { env_id: String },
    /// 下载文件
    Download { url: String, dest: PathBuf },
    /// 解压文件
    Extract { src: PathBuf, dest: PathBuf },
    /// 调用插件动作
    PluginAction { plugin_id: String, action: String, params: serde_json::Value },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum StepStatus {
    Pending,
    Waiting,   // 等待依赖完成
    Running,
    Success,
    Failed { error: String },
    Skipped,
    Cancelled,
}
```

### 5.3 任务 `Task`

```rust
// src-tauri/src/task/engine.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Task {
    pub id: String,
    pub name: String,
    pub recipe_id: String,
    pub status: TaskStatus,
    pub steps: Vec<TaskStep>,
    pub created_at: u64,
    pub started_at: Option<u64>,
    pub finished_at: Option<u64>,
    /// 总进度 0-100
    pub progress: f32,
    pub error_summary: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TaskStatus {
    Idle,
    Running,
    Paused,
    Success,
    Failed,
    Cancelled,
}
```

### 5.4 配方 `Recipe`

```rust
// src-tauri/src/recipe/schema.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Recipe {
    /// 配方格式版本，当前 "1"
    pub version: String,
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub author: Option<String>,
    pub tags: Vec<String>,
    /// 目标平台：["macos", "windows", "linux"] 或 ["*"]
    pub platforms: Vec<String>,
    /// 环境前置检查
    pub env_requirements: Vec<EnvRequirement>,
    /// 步骤定义
    pub steps: Vec<RecipeStep>,
    /// 配方级变量
    pub vars: HashMap<String, String>,
    /// 配方元数据
    pub metadata: RecipeMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvRequirement {
    pub env_id: String,
    pub version: Option<String>,  // semver 约束，如 ">=18.0.0"
    pub optional: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecipeStep {
    pub id: String,
    pub name: String,
    pub action: StepAction,
    pub depends_on: Vec<String>,
    /// 条件执行表达式（基于变量和环境状态）
    pub condition: Option<String>,
    pub retry: Option<RetryConfig>,
    pub timeout_secs: Option<u64>,
    pub on_error: OnErrorStrategy,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum OnErrorStrategy {
    #[default]
    Fail,
    Skip,
    Retry,
    /// 执行另一个步骤 ID
    RunStep(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    pub max_attempts: u8,
    pub delay_secs: u64,
    pub backoff: BackoffStrategy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BackoffStrategy {
    Fixed,
    Exponential,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeMetadata {
    pub created_at: Option<String>,
    pub source_url: Option<String>,
    pub checksum: Option<String>,
}
```

### 5.5 日志条目 `LogEntry`

```rust
// src-tauri/src/log/pipeline.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LogEntry {
    pub id: u64,
    pub task_id: String,
    pub step_id: Option<String>,
    pub level: LogLevel,
    pub message: String,
    pub timestamp: u64,   // Unix ms
    pub source: LogSource,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum LogSource {
    Stdout,
    Stderr,
    System,
    Plugin { plugin_id: String },
}
```

### 5.6 错误诊断 `DiagnosticReport`

```rust
// src-tauri/src/error/engine.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiagnosticReport {
    pub task_id: String,
    pub step_id: String,
    pub error_code: Option<String>,
    pub raw_error: String,
    pub matched_rule: Option<ErrorRule>,
    pub suggestions: Vec<FixSuggestion>,
    pub auto_fixable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ErrorRule {
    pub id: String,
    pub pattern: String,   // 正则表达式
    pub description: String,
    pub category: ErrorCategory,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FixSuggestion {
    pub title: String,
    pub description: String,
    pub action: Option<FixAction>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum FixAction {
    RunCommand { command: String, args: Vec<String> },
    RetryStep { step_id: String },
    InstallEnv { env_id: String },
    OpenUrl { url: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ErrorCategory {
    NetworkError,
    PermissionDenied,
    MissingDependency,
    VersionConflict,
    DiskSpace,
    Timeout,
    Unknown,
}
```

### 5.7 前端共享类型（TypeScript）

```typescript
// src/ipc/types.ts — 与 Rust 结构体对应的 TS 类型

export type EnvStatus =
  | { type: 'ok' }
  | { type: 'missing' }
  | { type: 'versionMismatch'; found: string; required: string }
  | { type: 'error'; message: string }
  | { type: 'checking' };

export interface EnvItem {
  id: string;
  name: string;
  status: EnvStatus;
  version?: string;
  requiredVersion?: string;
  path?: string;
  installHint?: { macos?: string; windows?: string; linux?: string };
  checkedAt: number;
}

export type StepStatus =
  | 'pending'
  | 'waiting'
  | 'running'
  | 'success'
  | { type: 'failed'; error: string }
  | 'skipped'
  | 'cancelled';

export interface TaskStep {
  id: string;
  name: string;
  description?: string;
  status: StepStatus;
  startedAt?: number;
  finishedAt?: number;
  exitCode?: number;
  retryCount: number;
  maxRetries: number;
}

export type TaskStatus =
  | 'idle' | 'running' | 'paused' | 'success' | 'failed' | 'cancelled';

export interface Task {
  id: string;
  name: string;
  recipeId: string;
  status: TaskStatus;
  steps: TaskStep[];
  createdAt: number;
  startedAt?: number;
  finishedAt?: number;
  progress: number;
  errorSummary?: string;
}

export interface LogEntry {
  id: number;
  taskId: string;
  stepId?: string;
  level: 'debug' | 'info' | 'warn' | 'error';
  message: string;
  timestamp: number;
  source: 'stdout' | 'stderr' | 'system' | { pluginId: string };
}

export interface DiagnosticReport {
  taskId: string;
  stepId: string;
  rawError: string;
  matchedRule?: ErrorRule;
  suggestions: FixSuggestion[];
  autoFixable: boolean;
}

export interface Recipe {
  version: string;
  id: string;
  name: string;
  description?: string;
  author?: string;
  tags: string[];
  platforms: string[];
  envRequirements: EnvRequirement[];
  steps: RecipeStep[];
  vars: Record<string, string>;
}
```

---

## 6. Rust 后端接口设计（Tauri Commands）

所有命令通过 `tauri::command` 注册，在 `src-tauri/src/commands/` 中实现。

### 6.1 环境检测命令

```rust
// src-tauri/src/commands/env_commands.rs

/// 检测所有注册的环境项
#[tauri::command]
pub async fn probe_all_envs(
    state: State<'_, AppState>,
) -> Result<Vec<EnvItem>, AppError> { ... }

/// 检测单个环境项
#[tauri::command]
pub async fn probe_env(
    state: State<'_, AppState>,
    env_id: String,
) -> Result<EnvItem, AppError> { ... }

/// 安装缺失的环境项（异步，通过事件推送进度）
#[tauri::command]
pub async fn install_env(
    app: AppHandle,
    state: State<'_, AppState>,
    env_id: String,
) -> Result<String, AppError> { ... }  // 返回 task_id
```

### 6.2 任务控制命令

```rust
// src-tauri/src/commands/task_commands.rs

/// 根据配方创建并启动任务
#[tauri::command]
pub async fn start_task(
    app: AppHandle,
    state: State<'_, AppState>,
    recipe_id: String,
    vars: HashMap<String, String>,
) -> Result<String, AppError> { ... }  // 返回 task_id

/// 暂停任务（等待当前步骤完成后暂停）
#[tauri::command]
pub async fn pause_task(
    state: State<'_, AppState>,
    task_id: String,
) -> Result<(), AppError> { ... }

/// 恢复暂停的任务
#[tauri::command]
pub async fn resume_task(
    state: State<'_, AppState>,
    task_id: String,
) -> Result<(), AppError> { ... }

/// 取消任务
#[tauri::command]
pub async fn cancel_task(
    state: State<'_, AppState>,
    task_id: String,
) -> Result<(), AppError> { ... }

/// 重试失败的步骤
#[tauri::command]
pub async fn retry_step(
    state: State<'_, AppState>,
    task_id: String,
    step_id: String,
) -> Result<(), AppError> { ... }

/// 获取任务当前快照
#[tauri::command]
pub async fn get_task(
    state: State<'_, AppState>,
    task_id: String,
) -> Result<Task, AppError> { ... }

/// 列出历史任务
#[tauri::command]
pub async fn list_tasks(
    state: State<'_, AppState>,
) -> Result<Vec<Task>, AppError> { ... }

/// 触发错误诊断
#[tauri::command]
pub async fn diagnose_error(
    state: State<'_, AppState>,
    task_id: String,
    step_id: String,
) -> Result<DiagnosticReport, AppError> { ... }

/// 执行修复建议
#[tauri::command]
pub async fn apply_fix(
    app: AppHandle,
    state: State<'_, AppState>,
    task_id: String,
    fix_action: FixAction,
) -> Result<(), AppError> { ... }
```

### 6.3 配方管理命令

```rust
// src-tauri/src/commands/recipe_commands.rs

/// 列出所有已安装配方
#[tauri::command]
pub async fn list_recipes(
    state: State<'_, AppState>,
) -> Result<Vec<Recipe>, AppError> { ... }

/// 加载配方文件（TOML 路径）
#[tauri::command]
pub async fn load_recipe_file(
    state: State<'_, AppState>,
    path: String,
) -> Result<Recipe, AppError> { ... }

/// 从 URL 拉取配方
#[tauri::command]
pub async fn fetch_recipe_url(
    state: State<'_, AppState>,
    url: String,
) -> Result<Recipe, AppError> { ... }

/// 校验配方合法性
#[tauri::command]
pub async fn validate_recipe(
    state: State<'_, AppState>,
    recipe: Recipe,
) -> Result<Vec<ValidationIssue>, AppError> { ... }

/// 保存/更新配方
#[tauri::command]
pub async fn save_recipe(
    state: State<'_, AppState>,
    recipe: Recipe,
) -> Result<(), AppError> { ... }

/// 删除配方
#[tauri::command]
pub async fn delete_recipe(
    state: State<'_, AppState>,
    recipe_id: String,
) -> Result<(), AppError> { ... }
```

### 6.4 Tauri 事件（后端 → 前端推送）

| 事件名 | Payload 类型 | 触发时机 |
|--------|------------|----------|
| `task://progress` | `TaskProgressEvent` | 任务进度变化 |
| `task://step-update` | `TaskStep` | 步骤状态变化 |
| `task://status` | `TaskStatusEvent` | 任务整体状态变化 |
| `log://entry` | `LogEntry` | 新日志条目 |
| `env://update` | `EnvItem` | 环境项状态更新 |
| `error://diagnostic` | `DiagnosticReport` | 错误诊断完成 |

```rust
// 事件 Payload 示例
#[derive(Clone, Serialize)]
pub struct TaskProgressEvent {
    pub task_id: String,
    pub progress: f32,
    pub current_step_id: Option<String>,
}

// 发送示例（在 TaskExecutor 内）
app_handle.emit("task://progress", TaskProgressEvent {
    task_id: task.id.clone(),
    progress: task.progress,
    current_step_id: current_step.map(|s| s.id.clone()),
})?;
```

---

## 7. 前端页面信息架构

### 7.1 仪表板页面（Dashboard）

```
┌─────────────────────────────────────────────────────┐
│  One Key OpenClaw         [通知] [设置]              │
├──────────┬──────────────────────────────────────────┤
│          │  ┌─────────────────────────────────────┐ │
│  侧边栏   │  │ 环境状态概览（EnvSummaryCard × N）  │ │
│          │  │  ✅ Node.js 20.x  ✅ Git 2.x        │ │
│  📊 仪表板│  │  ❌ Python 3.x   ⚠️  Docker        │ │
│  🔍 环境  │  └─────────────────────────────────────┘ │
│  🚀 部署  │  ┌──────────────────┐ ┌───────────────┐  │
│  📋 任务  │  │ 最近任务列表      │ │ 快速操作      │  │
│  📝 日志  │  │ ● OpenClaw 安装   │ │ [一键部署]    │  │
│  📦 配方  │  │ ✅ Claude Code    │ │ [检测环境]    │  │
│  ⚙️ 设置  │  └──────────────────┘ └───────────────┘  │
│          │                                            │
└──────────┴────────────────────────────────────────────┘
```

### 7.2 环境检测页面（EnvCheck）

```
环境检测                                    [全部重新检测]
─────────────────────────────────────────────────────────
搜索... [筛选: 全部 | 正常 | 缺失 | 版本不符]

┌─────────────────────────────────────────────────────┐
│ ✅ Node.js          已安装   v20.11.0  ≥18.0.0 ✓   │
│    /usr/local/bin/node                   [重新检测] │
├─────────────────────────────────────────────────────┤
│ ❌ Python           缺失     -          ≥3.10.0     │
│    建议: brew install python3           [立即安装] │
├─────────────────────────────────────────────────────┤
│ ⚠️ Docker           版本不符  v24.0.0   ≥25.0.0    │
│    建议: 升级 Docker Desktop            [查看说明] │
└─────────────────────────────────────────────────────┘
```

### 7.3 一键部署页面（Deploy）

```
选择配方 ▼ [OpenClaw 完整部署 v1.2.0]       [开始部署]

─── 前置检查 ─────────────────────────────────────────
  ✅ Node.js ≥18   ✅ Git   ⚠️ Python（可选）

─── 步骤进度 ─────────────────────────────────────────
  ✅ 1. 克隆仓库           完成  0:08
  🔄 2. 安装 npm 依赖      进行中  ████████░░  79%
  ⏳ 3. 构建前端
  ⏳ 4. 配置环境变量
  ⏳ 5. 启动服务

─── 实时日志 ─────────────────────────────────────────
  [终端窗口，可展开/折叠]
  added 842 packages in 12s

[暂停]  [取消]                           总进度: 38%
```

### 7.4 任务编排可视化页面（TaskFlow）

```
                ┌─────────┐
                │ 克隆仓库 │
                └────┬────┘
                     │
          ┌──────────┴──────────┐
          ▼                     ▼
    ┌──────────┐          ┌──────────┐
    │ 安装依赖  │          │ 下载工具 │
    └────┬─────┘          └────┬─────┘
          └──────────┬──────────┘
                     ▼
              ┌─────────────┐
              │  构建 & 启动 │
              └─────────────┘
```

### 7.5 日志终端页面（Log）

```
[Task: OpenClaw 安装]  [步骤: 安装依赖]  [级别: 全部 ▼]   [导出]
─────────────────────────────────────────────────────────────────
 14:23:01  INFO   npm install --production
 14:23:02  STDOUT added 842 packages in 12s
 14:23:02  WARN   deprecated package: lodash@3.x
 14:23:05  ERROR  EACCES: permission denied /usr/local/lib
─────────────────────────────────────────────────────────────────
[自动滚动 ✓]  [清空]  [搜索: ______]
```

### 7.6 配方管理页面（Recipe）

```
配方库                                    [导入文件] [从 URL 导入]
─────────────────────────────────────────────────────────────────
┌─────────────────────────────────────────────────────────────┐
│ 📦 OpenClaw 完整部署          作者: official  v1.2.0        │
│    macOS / Linux / Windows   标签: openclaw deploy          │
│    [查看详情]  [编辑]  [运行]                    [删除]     │
├─────────────────────────────────────────────────────────────┤
│ 📦 Claude Code 快速安装       作者: community  v0.9.1       │
│    macOS / Linux              标签: claude ai               │
│    [查看详情]  [编辑]  [运行]                    [删除]     │
└─────────────────────────────────────────────────────────────┘

[新建配方]
```

---

## 8. 任务状态机

### 8.1 任务级状态机

```
                    ┌─────────────────┐
                    │      Idle       │
                    └────────┬────────┘
                             │ start_task()
                    ┌────────▼────────┐
              ┌────►│    Running      │◄────┐
              │     └──┬──┬──┬───────┘     │
              │        │  │  │              │ resume_task()
    pause_    │        │  │  │ pause_task() │
    resolved  │        │  │  └─────────────►│
              │        │  │            ┌────┴────┐
              │        │  └───────────►│ Paused  │
              │        │  cancel_task()└─────────┘
              │        │
              │  ┌─────▼─────┐    ┌─────────────┐
              │  │  Success  │    │   Failed    │
              │  └─────┬─────┘    └──────┬──────┘
              │        │                  │ retry / 
              └────────┘                  │ diagnose
                                   ┌──────▼──────┐
                                   │  Cancelled  │
                                   └─────────────┘
```

### 8.2 步骤级状态机

```
Pending ──► Waiting ──► Running ──► Success
                │           │
                │           └──► Failed ──► (retry) ──► Running
                │                    │
                │                    └──► (on_error: skip) ──► Skipped
                │
                └──► Cancelled
```

### 8.3 状态机实现（Rust）

```rust
// src-tauri/src/task/state_machine.rs
pub struct TaskStateMachine {
    task: Arc<RwLock<Task>>,
    control_tx: mpsc::Sender<TaskControl>,
}

pub enum TaskControl {
    Pause,
    Resume,
    Cancel,
    RetryStep(String),
}

impl TaskStateMachine {
    pub fn transition(&self, event: TaskEvent) -> Result<(), AppError> {
        let mut task = self.task.write().unwrap();
        let new_status = match (&task.status, &event) {
            (TaskStatus::Idle,    TaskEvent::Start)   => TaskStatus::Running,
            (TaskStatus::Running, TaskEvent::Pause)   => TaskStatus::Paused,
            (TaskStatus::Paused,  TaskEvent::Resume)  => TaskStatus::Running,
            (TaskStatus::Running, TaskEvent::Complete)=> TaskStatus::Success,
            (TaskStatus::Running, TaskEvent::Fail)    => TaskStatus::Failed,
            (_, TaskEvent::Cancel) => TaskStatus::Cancelled,
            _ => return Err(AppError::InvalidStateTransition {
                from: task.status.clone(),
                event,
            }),
        };
        task.status = new_status;
        Ok(())
    }
}
```

---

## 9. 错误处理机制

### 9.1 错误类型层次

```rust
// src-tauri/src/error/mod.rs
#[derive(Debug, thiserror::Error, Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum AppError {
    #[error("环境检测失败: {env_id} - {message}")]
    EnvProbeError { env_id: String, message: String },

    #[error("步骤执行失败: {step_id} (exit code {exit_code:?})")]
    StepExecutionError { step_id: String, exit_code: Option<i32>, stderr: String },

    #[error("配方解析错误: {path} - {message}")]
    RecipeParseError { path: String, message: String },

    #[error("配方校验失败: {issues:?}")]
    RecipeValidationError { issues: Vec<ValidationIssue> },

    #[error("状态转换非法: {from:?} + {event:?}")]
    InvalidStateTransition { from: TaskStatus, event: TaskEvent },

    #[error("插件错误: {plugin_id} - {message}")]
    PluginError { plugin_id: String, message: String },

    #[error("网络错误: {message}")]
    NetworkError { message: String },

    #[error("IO 错误: {0}")]
    IoError(#[from] std::io::Error),

    #[error("系统错误: {0}")]
    Anyhow(#[from] anyhow::Error),
}

// 实现 Tauri IPC 序列化
impl serde::Serialize for AppError { ... }
```

### 9.2 错误诊断引擎

```rust
// src-tauri/src/error/engine.rs
pub struct ErrorDiagnosticEngine {
    rules: Vec<ErrorRule>,
}

impl ErrorDiagnosticEngine {
    pub fn diagnose(&self, error: &AppError, context: &ExecutionContext) -> DiagnosticReport {
        let raw_error = error.to_string();

        // 1. 匹配规则
        let matched_rule = self.rules.iter().find(|rule| {
            Regex::new(&rule.pattern).map(|re| re.is_match(&raw_error)).unwrap_or(false)
        });

        // 2. 生成修复建议
        let suggestions = self.build_suggestions(matched_rule, context);
        let auto_fixable = suggestions.iter().any(|s| s.action.is_some());

        DiagnosticReport {
            task_id: context.task_id.clone(),
            step_id: context.step_id.clone(),
            raw_error,
            matched_rule: matched_rule.cloned(),
            suggestions,
            auto_fixable,
        }
    }
}
```

### 9.3 内置错误规则示例（TOML）

```toml
# src-tauri/assets/error_rules.toml

[[rules]]
id = "npm_eacces"
pattern = "EACCES: permission denied"
description = "npm 安装权限不足"
category = "PermissionDenied"

[[rules.suggestions]]
title = "使用 sudo 重试"
action = { type = "runCommand", command = "sudo", args = ["npm", "install"] }

[[rules.suggestions]]
title = "修复 npm 全局目录权限"
action = { type = "openUrl", url = "https://docs.npmjs.com/resolving-eacces-permissions-errors-when-installing-packages-globally" }

[[rules]]
id = "network_timeout"
pattern = "(ETIMEDOUT|ENOTFOUND|getaddrinfo)"
description = "网络连接超时或 DNS 解析失败"
category = "NetworkError"

[[rules.suggestions]]
title = "检查网络连接并重试"
action = { type = "retryStep", stepId = "__current__" }

[[rules.suggestions]]
title = "配置代理"
action = { type = "runCommand", command = "npm", args = ["config", "set", "proxy", "http://127.0.0.1:7890"] }
```

---

## 10. 配方系统设计（Recipe System）

### 10.1 配方文件格式（TOML）

```toml
# recipes/openclaw-full.toml

version = "1"
id = "openclaw-full"
name = "OpenClaw 完整部署"
description = "一键克隆、构建并启动 OpenClaw 服务"
author = "official"
tags = ["openclaw", "deploy", "full"]
platforms = ["macos", "linux", "windows"]

[vars]
REPO_URL = "https://github.com/example/openclaw"
INSTALL_DIR = "~/.openclaw"
NODE_ENV = "production"

[[env_requirements]]
env_id = "node"
version = ">=18.0.0"
optional = false

[[env_requirements]]
env_id = "git"
optional = false

[[env_requirements]]
env_id = "python"
version = ">=3.10.0"
optional = true

# ─── 步骤定义 ────────────────────────────────────────

[[steps]]
id = "clone"
name = "克隆仓库"
[steps.action]
type = "shell"
command = "git"
args = ["clone", "${REPO_URL}", "${INSTALL_DIR}"]

[[steps]]
id = "install-deps"
name = "安装 Node.js 依赖"
depends_on = ["clone"]
[steps.action]
type = "shell"
command = "npm"
args = ["install", "--production"]
env = { NODE_ENV = "${NODE_ENV}" }
[steps.retry]
max_attempts = 3
delay_secs = 5
backoff = "Exponential"
on_error = "Retry"

[[steps]]
id = "build-frontend"
name = "构建前端"
depends_on = ["install-deps"]
[steps.action]
type = "shell"
command = "npm"
args = ["run", "build"]
timeout_secs = 300

[[steps]]
id = "start-service"
name = "启动服务"
depends_on = ["build-frontend"]
[steps.action]
type = "shell"
command = "npm"
args = ["start"]
```

### 10.2 配方注册表

```rust
// src-tauri/src/recipe/registry.rs
pub struct RecipeRegistry {
    /// 已加载的配方（id → Recipe）
    recipes: HashMap<String, Recipe>,
    /// 本地配方目录
    local_dir: PathBuf,
    /// 配方文件监听器（热重载）
    watcher: Option<notify::RecommendedWatcher>,
}

impl RecipeRegistry {
    /// 扫描本地配方目录并加载所有 .toml 配方
    pub async fn load_local_recipes(&mut self) -> Result<(), AppError> { ... }

    /// 从远程 URL 拉取配方
    pub async fn fetch_from_url(&mut self, url: &str) -> Result<Recipe, AppError> { ... }

    /// 订阅文件变更，热重载配方
    pub fn watch_local_dir(&mut self, app: AppHandle) -> Result<(), AppError> { ... }

    /// 校验配方
    pub fn validate(&self, recipe: &Recipe) -> Vec<ValidationIssue> { ... }
}
```

### 10.3 配方 → 任务转换

```rust
// src-tauri/src/recipe/parser.rs
impl Recipe {
    /// 将配方转换为可执行任务
    pub fn into_task(self, vars: HashMap<String, String>) -> Result<Task, AppError> {
        // 1. 合并变量（配方默认 + 用户传入）
        let resolved_vars = self.resolve_vars(vars);

        // 2. 将 RecipeStep → TaskStep（变量替换、条件求值）
        let steps: Vec<TaskStep> = self.steps.iter()
            .filter(|s| self.eval_condition(s.condition.as_deref(), &resolved_vars))
            .map(|s| s.into_task_step(&resolved_vars))
            .collect::<Result<Vec<_>, _>>()?;

        // 3. 构建 DAG 并校验无环
        let graph = TaskGraph::build(&steps)?;

        Ok(Task {
            id: uuid::Uuid::new_v4().to_string(),
            name: self.name,
            recipe_id: self.id,
            status: TaskStatus::Idle,
            steps,
            created_at: now_millis(),
            ..Default::default()
        })
    }
}
```

---

## 11. 插件式扩展架构

### 11.1 插件类型

| 插件类型 | 描述 | 示例 |
|----------|------|------|
| `recipe_provider` | 提供配方列表 | 社区配方市场 |
| `env_probe` | 自定义环境检测 | Docker 镜像检测 |
| `step_executor` | 自定义步骤执行器 | Kubernetes 部署 |
| `error_rule` | 扩展错误规则 | 云平台错误码 |
| `log_sink` | 日志输出目标 | 发送到 Loki/Elasticsearch |

### 11.2 插件定义

```rust
// src-tauri/src/plugin/api.rs
pub trait RecipeProviderPlugin: Send + Sync {
    fn id(&self) -> &str;
    fn name(&self) -> &str;
    async fn list_recipes(&self) -> Result<Vec<Recipe>, AppError>;
}

pub trait EnvProbePlugin: Send + Sync {
    fn env_id(&self) -> &str;
    async fn probe(&self) -> Result<EnvItem, AppError>;
}

pub trait StepExecutorPlugin: Send + Sync {
    fn action_type(&self) -> &str;
    async fn execute(
        &self,
        step: &TaskStep,
        context: &ExecutionContext,
        log_tx: mpsc::Sender<LogEntry>,
    ) -> Result<(), AppError>;
}
```

### 11.3 插件清单文件（plugin.toml）

```toml
# plugins/my-plugin/plugin.toml
id = "my-k8s-plugin"
name = "Kubernetes 部署插件"
version = "1.0.0"
author = "contributor"
types = ["step_executor"]
entry = "plugin.wasm"   # WebAssembly 模块（沙箱化执行）

[permissions]
network = true
filesystem = false
```

### 11.4 插件管理器

```rust
// src-tauri/src/plugin/manager.rs
pub struct PluginManager {
    plugins: HashMap<String, Box<dyn StepExecutorPlugin>>,
    env_probes: HashMap<String, Box<dyn EnvProbePlugin>>,
    plugin_dir: PathBuf,
}

impl PluginManager {
    pub async fn load_plugin(&mut self, path: &Path) -> Result<(), AppError> { ... }
    pub async fn unload_plugin(&mut self, id: &str) -> Result<(), AppError> { ... }
    pub fn get_executor(&self, action_type: &str) -> Option<&dyn StepExecutorPlugin> { ... }
}
```

---

## 12. 多平台适配方案

### 12.1 平台抽象层（HAL）

```rust
// src-tauri/src/env/platform.rs
pub trait PlatformHAL: Send + Sync {
    fn os(&self) -> TargetOS;
    fn default_shell(&self) -> (&str, &[&str]);   // (cmd, args_prefix)
    fn package_managers(&self) -> Vec<PackageManager>;
    fn home_dir(&self) -> PathBuf;
    fn app_data_dir(&self) -> PathBuf;
    fn path_separator(&self) -> char;
}

pub struct MacOSPlatform;
pub struct WindowsPlatform;
pub struct LinuxPlatform;

// 运行时选择
pub fn current_platform() -> Box<dyn PlatformHAL> {
    #[cfg(target_os = "macos")]  { Box::new(MacOSPlatform) }
    #[cfg(target_os = "windows")]{ Box::new(WindowsPlatform) }
    #[cfg(target_os = "linux")]  { Box::new(LinuxPlatform) }
}
```

### 12.2 包管理器适配

| 平台 | 默认包管理器 | 备选 |
|------|------------|------|
| macOS | Homebrew (`brew`) | MacPorts |
| Windows | winget | Chocolatey, Scoop |
| Linux (Debian) | apt | snap |
| Linux (RHEL) | dnf | yum |
| 跨平台 | npm / pip / cargo | — |

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PackageManager {
    Brew, Winget, Chocolatey, Scoop,
    Apt, Dnf, Pacman, Snap,
    Npm, Pip, Cargo,
}

impl PackageManager {
    pub fn install_command(&self, packages: &[String]) -> (String, Vec<String>) {
        match self {
            Self::Brew    => ("brew".into(), vec!["install".into()].into_iter().chain(packages.iter().cloned()).collect()),
            Self::Winget  => ("winget".into(), vec!["install".into()].into_iter().chain(packages.iter().cloned()).collect()),
            Self::Apt     => ("apt-get".into(), vec!["install".into(), "-y".into()].into_iter().chain(packages.iter().cloned()).collect()),
            Self::Npm     => ("npm".into(), vec!["install".into(), "-g".into()].into_iter().chain(packages.iter().cloned()).collect()),
            // ...
        }
    }
}
```

### 12.3 配方平台条件

```toml
# 配方中使用平台条件步骤
[[steps]]
id = "install-brew"
name = "安装 Homebrew（仅 macOS）"
condition = "platform == 'macos'"
[steps.action]
type = "shell"
command = "/bin/bash"
args = ["-c", "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"]

[[steps]]
id = "install-winget-deps"
name = "安装 Windows 依赖"
condition = "platform == 'windows'"
[steps.action]
type = "packageInstall"
manager = "Winget"
packages = ["Git.Git", "OpenJS.NodeJS.LTS"]
```

---

## 13. 实时日志流架构

### 13.1 日志管道

```
子进程 stdout/stderr
       │
       ▼
  LogPipeline (Rust)
  ├── 解析行
  ├── 附加 metadata（task_id, step_id, level, timestamp）
  ├── 推送到 mpsc::channel
  │
  ▼
LogBroadcaster
  ├── 持久化到 SQLite（可选）
  └── app_handle.emit("log://entry", entry)
              │
              ▼
        前端 useTauriEvent("log://entry")
              │
              ▼
        logStore (Zustand) ── 环形缓冲区（最近 10000 条）
              │
              ▼
        LogTerminal (Xterm.js)
```

### 13.2 日志流实现

```rust
// src-tauri/src/log/pipeline.rs
pub async fn stream_command(
    app: AppHandle,
    task_id: String,
    step_id: String,
    mut child: tokio::process::Child,
) -> Result<ExitStatus, AppError> {
    let stdout = child.stdout.take().expect("stdout");
    let stderr = child.stderr.take().expect("stderr");

    let app_clone = app.clone();
    let task_id_clone = task_id.clone();
    let step_id_clone = step_id.clone();

    // 并发读取 stdout 和 stderr
    let stdout_task = tokio::spawn(async move {
        let mut reader = BufReader::new(stdout).lines();
        let mut seq = 0u64;
        while let Some(line) = reader.next_line().await? {
            app_clone.emit("log://entry", LogEntry {
                id: { seq += 1; seq },
                task_id: task_id_clone.clone(),
                step_id: Some(step_id_clone.clone()),
                level: LogLevel::Info,
                message: line,
                timestamp: now_millis(),
                source: LogSource::Stdout,
            })?;
        }
        Ok::<_, AppError>(())
    });

    let stderr_task = tokio::spawn(/* 类似，source: Stderr, level: Warn */);

    let status = child.wait().await?;
    let _ = tokio::join!(stdout_task, stderr_task);
    Ok(status)
}
```

### 13.3 前端日志订阅

```typescript
// src/hooks/useLogStream.ts
export function useLogStream(taskId: string) {
  const addLog = useLogStore(s => s.addLog);

  useEffect(() => {
    const unlisten = listen<LogEntry>('log://entry', (event) => {
      if (event.payload.taskId === taskId) {
        addLog(event.payload);
      }
    });
    return () => { unlisten.then(fn => fn()); };
  }, [taskId, addLog]);
}
```

---

## 14. 目录结构

```
One_Key_OpenClaw/
├── src-tauri/
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   ├── capabilities/
│   │   └── default.json           # Tauri 2 权限声明
│   ├── assets/
│   │   ├── error_rules.toml       # 内置错误规则
│   │   └── env_probes.toml        # 内置环境探测配置
│   └── src/
│       ├── main.rs
│       ├── lib.rs
│       ├── state.rs               # AppState 定义
│       ├── env/
│       ├── recipe/
│       ├── task/
│       ├── error/
│       ├── log/
│       ├── plugin/
│       └── commands/
├── src/
│   ├── main.tsx
│   ├── App.tsx
│   ├── pages/
│   ├── components/
│   ├── store/
│   ├── hooks/
│   ├── ipc/
│   └── utils/
├── recipes/
│   ├── openclaw-full.toml         # OpenClaw 完整部署配方
│   ├── claude-code-install.toml   # Claude Code 快速安装配方
│   └── README.md
├── plugins/                       # 可选插件目录
├── package.json
├── tsconfig.json
├── tailwind.config.ts
├── vite.config.ts
├── task.md                        # 本文档
└── README.md
```

---

## 15. 开发路线图

### Phase 1 — 基础骨架（MVP）

- [x] 项目初始化：`pnpm create tauri-app`（Tauri 2 + React + TypeScript）
- [x] TailwindCSS + Radix UI 集成
- [x] Zustand 状态管理基础结构
- [x] `EnvProber`：检测 Node.js、Git、Python、Rust、Docker
- [x] 环境检测页面（EnvCheckPage）
- [x] Tauri 事件系统 + `LogPipeline` 基础版
- [x] 日志终端页面（LogPage + Xterm.js）

### Phase 2 — 配方与任务引擎

- [x] `RecipeParser`：TOML 配方解析与校验
- [x] `TaskExecutor`：串行步骤执行（无 DAG）
- [x] `TaskStateMachine`：状态转换 + 暂停/恢复/取消
- [x] 一键部署页面（DeployPage）
- [x] 分步骤进度展示（StepList）
- [x] 内置 OpenClaw 与 Claude Code 配方

### Phase 3 — 高级编排与错误处理

- [x] DAG 任务图（petgraph）：并行步骤支持
- [x] 任务编排可视化页面（TaskFlowPage + React Flow）
- [x] `ErrorDiagnosticEngine`：规则匹配 + 修复建议
- [x] 错误诊断面板（ErrorAlert + FixSuggestion 列表）
- [x] 步骤级重试（RetryConfig）
- [x] 仪表板页面（DashboardPage）

### Phase 4 — 插件与扩展

- [x] `PluginManager`：本地插件加载
- [x] 插件 API 规范文档
- [x] `RecipeRegistry`：热重载 + 远程拉取
- [x] 配方管理页面（RecipePage + RecipeEditor）
- [x] 配方市场原型（社区配方列表）

### Phase 5 — 打磨与发布

- [x] 完整多平台测试（macOS / Windows / Linux）
- [x] 自动更新（tauri-plugin-updater）
- [x] 国际化（i18n，中/英）
- [x] 单元测试（Rust）+ E2E 测试（Playwright）
- [x] 打包 & CI/CD（GitHub Actions）
- [x] 文档站点

---

*文档版本: 1.0.0 | 生成时间: 2026-03-09*
