# One Key OpenClaw

一键部署工具，基于 Tauri 2 + React + TypeScript 构建的跨平台桌面应用。

A one-click deployment tool built with Tauri 2 + React + TypeScript.

---

## ✨ Features / 功能特性

- 🔍 **Environment Detection** — Auto-detect Node.js, Git, Python, Rust, Docker
- 📦 **Recipe Management** — TOML-based declarative deployment recipes
- 🚀 **One-Key Deploy** — Execute multi-step deployments with a single click
- 📋 **DAG Task Flow** — Visualize and execute tasks with dependency graphs
- 🔌 **Plugin System** — Extensible plugin architecture for custom recipes, probes, and error rules
- 🛡️ **Error Diagnostics** — Intelligent error matching with auto-fix suggestions
- 📝 **Log Viewer** — Real-time terminal-based log viewer
- 🌐 **i18n** — Chinese/English bilingual interface
- 🔄 **Auto-Update** — Built-in update mechanism via tauri-plugin-updater

## 🏗️ Architecture / 架构

```
┌───────────────────────────────────────────┐
│              Frontend (React)             │
│  ┌─────────┐ ┌──────────┐ ┌───────────┐  │
│  │ Zustand  │ │  Pages   │ │Components │  │
│  │ Stores   │ │ (Router) │ │ (UI)      │  │
│  └────┬─────┘ └────┬─────┘ └────┬──────┘  │
│       └─────────┬──┘            │         │
│            ┌────▼────┐          │         │
│            │  IPC    │◄─────────┘         │
│            │ (Tauri) │                    │
│            └────┬────┘                    │
├─────────────────┼─────────────────────────┤
│            ┌────▼────┐  Backend (Rust)    │
│            │Commands │                    │
│            └────┬────┘                    │
│    ┌────────────┼────────────┐            │
│ ┌──▼──┐  ┌─────▼─────┐  ┌──▼──┐         │
│ │ Env │  │Recipe/Task │  │Error│         │
│ │Probe│  │  Engine    │  │Diag │         │
│ └─────┘  └───────────┘  └─────┘         │
│               ┌──▼──┐                    │
│               │Plugin│                    │
│               │Mgr   │                    │
│               └─────┘                    │
└───────────────────────────────────────────┘
```

## 📦 Quick Start / 快速开始

### Prerequisites / 前置条件

- [Node.js](https://nodejs.org/) ≥ 18
- [pnpm](https://pnpm.io/) ≥ 8
- [Rust](https://rustup.rs/) (stable)
- System dependencies:
  - **Linux**: `libgtk-3-dev libwebkit2gtk-4.1-dev libayatana-appindicator3-dev librsvg2-dev libjavascriptcoregtk-4.1-dev`
  - **macOS**: Xcode Command Line Tools
  - **Windows**: WebView2 (pre-installed on Windows 11)

### Development / 开发

```bash
# Install dependencies / 安装依赖
pnpm install

# Start dev server / 启动开发服务器
pnpm tauri dev

# Run Rust tests / 运行 Rust 测试
cd src-tauri && cargo test

# Run E2E tests / 运行 E2E 测试
pnpm test:e2e

# Build for production / 构建生产版本
pnpm tauri build
```

### Project Structure / 项目结构

```
├── src/                    # Frontend (React + TypeScript)
│   ├── components/         # Reusable UI components
│   ├── pages/              # Route pages
│   ├── store/              # Zustand state stores
│   ├── hooks/              # Custom React hooks
│   ├── ipc/                # Tauri IPC communication
│   ├── locales/            # i18n translation files (zh/en)
│   └── i18n.ts             # i18n configuration
├── src-tauri/              # Backend (Rust)
│   ├── src/
│   │   ├── commands/       # Tauri IPC command handlers
│   │   ├── env/            # Environment probing
│   │   ├── recipe/         # Recipe schema, parser, registry
│   │   ├── task/           # Task execution engine (DAG)
│   │   ├── error/          # Error diagnostics
│   │   ├── plugin/         # Plugin system
│   │   └── log/            # Logging
│   └── assets/             # Embedded assets (error rules, recipes)
├── e2e/                    # Playwright E2E tests
├── docs/                   # Documentation
└── .github/workflows/      # CI/CD pipelines
```

## 📖 Documentation / 文档

- [Architecture Guide](docs/architecture.md)
- [Plugin API Reference](docs/plugin-api.md)
- [Recipe Format Guide](docs/recipe-format.md)
- [Contributing Guide](docs/contributing.md)

## 🔧 CI/CD

The project uses GitHub Actions for:

- **CI** (`ci.yml`): Runs on every push/PR to `main`
  - Frontend type checking (`tsc`)
  - Rust tests on Linux, macOS, Windows
  - Multi-platform Tauri builds
- **Release** (`release.yml`): Triggered by version tags (`v*`)
  - Builds for all platforms
  - Creates GitHub release with binaries

## 📄 License

MIT
