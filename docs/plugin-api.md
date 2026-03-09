# One Key OpenClaw — 插件 API 规范 (Plugin API Specification)

> 版本: 1.0.0

---

## 1. 概述

One Key OpenClaw 支持通过**本地插件**扩展核心功能。每个插件是一个目录，包含
`plugin.toml` 清单文件和可选的入口模块。

插件目录默认位于应用可执行文件同级的 `plugins/` 目录下：

```
plugins/
├── my-k8s-plugin/
│   └── plugin.toml
├── docker-probe/
│   └── plugin.toml
└── ...
```

---

## 2. 插件清单文件 (plugin.toml)

每个插件目录**必须**包含一个 `plugin.toml`，格式如下：

```toml
# 插件唯一标识（必填）
id = "my-k8s-plugin"

# 人类可读的插件名称（必填）
name = "Kubernetes 部署插件"

# 语义化版本号（必填）
version = "1.0.0"

# 作者 / 维护者（可选）
author = "contributor"

# 插件能力类型列表（至少一项）
types = ["step_executor"]

# 入口文件（预留，用于未来 WASM 沙箱执行）
entry = "plugin.wasm"

# 权限声明
[permissions]
network = true        # 是否允许网络访问
filesystem = false    # 是否允许文件系统访问
```

---

## 3. 插件类型 (Plugin Types)

| 类型              | 标识              | 描述               | 示例               |
|-------------------|-------------------|--------------------|--------------------|
| 配方提供者         | `recipe_provider` | 提供配方列表        | 社区配方市场        |
| 环境探测          | `env_probe`       | 自定义环境检测      | Docker 镜像检测     |
| 步骤执行器         | `step_executor`   | 自定义步骤执行处理   | Kubernetes 部署     |
| 错误规则          | `error_rule`      | 扩展错误诊断规则    | 云平台错误码        |
| 日志输出          | `log_sink`        | 日志输出目标        | 发送到 Loki / ES    |

一个插件可以声明多种类型：

```toml
types = ["step_executor", "env_probe"]
```

---

## 4. 插件 API Traits（Rust）

### 4.1 RecipeProviderPlugin

```rust
pub trait RecipeProviderPlugin: Send + Sync {
    /// 插件唯一标识
    fn id(&self) -> &str;
    /// 人类可读名称
    fn name(&self) -> &str;
    /// 列出该插件提供的所有配方
    async fn list_recipes(&self) -> Result<Vec<Recipe>, AppError>;
}
```

### 4.2 EnvProbePlugin

```rust
pub trait EnvProbePlugin: Send + Sync {
    /// 要探测的环境项 ID
    fn env_id(&self) -> &str;
    /// 执行探测，返回环境项信息
    async fn probe(&self) -> Result<EnvItem, AppError>;
}
```

### 4.3 StepExecutorPlugin

```rust
pub trait StepExecutorPlugin: Send + Sync {
    /// 该执行器处理的 action type 名称
    fn action_type(&self) -> &str;
    /// 执行步骤
    async fn execute(
        &self,
        step: &TaskStep,
        context: &ExecutionContext,
        log_tx: mpsc::Sender<LogEntry>,
    ) -> Result<(), AppError>;
}
```

---

## 5. 插件管理器 (PluginManager)

`PluginManager` 负责插件的生命周期管理：

| 方法 | 描述 |
|------|------|
| `scan_plugins()` | 扫描插件目录，加载所有有效插件清单 |
| `load_plugin(path)` | 加载指定路径的单个插件 |
| `unload_plugin(id)` | 卸载指定 ID 的插件 |
| `list_plugins()` | 列出所有已加载插件的信息 |
| `get_plugin(id)` | 获取指定插件的详情 |

### Tauri Commands

| 命令 | 参数 | 返回值 |
|------|------|--------|
| `list_plugins` | — | `PluginInfo[]` |
| `load_plugin` | `path: string` | `PluginInfo` |
| `unload_plugin` | `pluginId: string` | `void` |
| `scan_plugins` | — | `number`（已加载数量） |

---

## 6. 前端类型定义 (TypeScript)

```typescript
type PluginType =
  | "recipe_provider"
  | "env_probe"
  | "step_executor"
  | "error_rule"
  | "log_sink";

interface PluginPermissions {
  network: boolean;
  filesystem: boolean;
}

interface PluginInfo {
  id: string;
  name: string;
  version: string;
  author?: string;
  types: PluginType[];
  enabled: boolean;
}
```

---

## 7. 权限模型

插件通过 `[permissions]` 表声明所需权限。默认所有权限关闭。

| 权限 | 默认 | 描述 |
|------|------|------|
| `network` | `false` | 允许发起网络请求 |
| `filesystem` | `false` | 允许读写文件系统 |

未来版本将通过 WASM 沙箱强制执行权限边界。

---

## 8. 开发插件

### 8.1 目录结构

```
my-plugin/
├── plugin.toml    # 插件清单（必须）
└── README.md      # 说明文档（推荐）
```

### 8.2 最小示例

```toml
id = "hello-plugin"
name = "Hello Plugin"
version = "0.1.0"
types = ["log_sink"]

[permissions]
network = false
filesystem = false
```

### 8.3 安装插件

将插件目录复制到 `plugins/` 下，然后在应用中调用"扫描插件"即可。

---

*文档版本: 1.0.0*
