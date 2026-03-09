# Contributing Guide / 贡献指南

Thank you for your interest in contributing to One Key OpenClaw!

感谢您对 One Key OpenClaw 的贡献！

## Development Setup / 开发环境

### Prerequisites / 前置条件

1. **Node.js** ≥ 18 and **pnpm** ≥ 8
2. **Rust** (stable toolchain via [rustup](https://rustup.rs/))
3. **System Dependencies**:

```bash
# Ubuntu / Debian
sudo apt install libgtk-3-dev libwebkit2gtk-4.1-dev \
  libayatana-appindicator3-dev librsvg2-dev \
  libjavascriptcoregtk-4.1-dev

# macOS — install Xcode Command Line Tools
xcode-select --install

# Windows — WebView2 is pre-installed on Windows 11
```

### Getting Started / 快速开始

```bash
# Clone the repository
git clone https://github.com/Ray-315/One_Key_OpenClaw.git
cd One_Key_OpenClaw

# Install frontend dependencies
pnpm install

# Start development server
pnpm tauri dev
```

## Project Structure / 项目结构

- **Frontend** (`src/`): React + TypeScript + TailwindCSS
- **Backend** (`src-tauri/`): Rust + Tauri 2
- **Tests**: Rust unit tests in `src-tauri/src/`, E2E tests in `e2e/`
- **Docs** (`docs/`): Project documentation

## Running Tests / 运行测试

```bash
# Rust unit tests
cd src-tauri && cargo test

# TypeScript type checking
pnpm build

# E2E tests (requires dev server or built app)
pnpm test:e2e
```

## Code Style / 代码风格

### Rust

- Use `cargo fmt` for formatting
- Use `cargo clippy` for linting
- Follow standard Rust naming conventions

### TypeScript

- Use TypeScript strict mode
- Use functional React components with hooks
- Use Zustand for state management (one store per domain)

## Adding a New Feature / 添加新功能

### Backend Command

1. Create the handler function in the appropriate `commands/` module
2. Register it in `lib.rs` via `generate_handler!`
3. Add types to the frontend `ipc/types.ts`

### Frontend Page

1. Create the page component in `src/pages/`
2. Add the route in `src/App.tsx`
3. Add the sidebar entry in `src/components/common/Sidebar.tsx`
4. Add i18n keys to `src/locales/{zh,en}.json`

### Recipe Step Action

1. Add the variant to `StepAction` enum in `src-tauri/src/recipe/schema.rs`
2. Add execution logic in `src-tauri/src/task/engine.rs`
3. Add validation in `validate_recipe()` if needed

## Pull Request Process / PR 流程

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/my-feature`)
3. Make your changes with tests
4. Ensure all tests pass (`cargo test` + `pnpm build`)
5. Submit a pull request

## i18n / 国际化

When adding or modifying UI text:

1. Add translation keys to both `src/locales/zh.json` and `src/locales/en.json`
2. Use `useTranslation()` hook in components:

```tsx
import { useTranslation } from "react-i18next";

function MyComponent() {
  const { t } = useTranslation();
  return <h1>{t("my.key")}</h1>;
}
```
