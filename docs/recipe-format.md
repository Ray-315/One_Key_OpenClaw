# Recipe Format Guide / 配方格式指南

## Overview / 概述

Recipes are TOML files that describe a deployment workflow. Each recipe contains metadata, environment requirements, variables, and an ordered list of steps.

## Recipe Structure / 配方结构

```toml
# Required: format version
version = "1"

# Required: unique identifier
id = "my-recipe"

# Required: human-readable name
name = "My Deployment Recipe"

# Optional fields
description = "Deploy my awesome project"
author = "Your Name"
tags = ["nodejs", "web"]
platforms = ["macos", "linux", "windows"]  # or ["*"] for all

# Environment requirements (optional)
[[env_requirements]]
env_id = "node"
version = ">=18.0.0"
optional = false

[[env_requirements]]
env_id = "git"

# Variables for substitution (optional)
[vars]
repo_url = "https://github.com/user/project.git"
install_dir = "/opt/my-project"

# Steps (required, at least one)
[[steps]]
id = "check-node"
name = "Check Node.js"
[steps.action]
type = "envCheck"
env_id = "node"

[[steps]]
id = "clone"
name = "Clone Repository"
depends_on = ["check-node"]
[steps.action]
type = "shell"
command = "git"
args = ["clone", "${repo_url}", "${install_dir}"]

[[steps]]
id = "install"
name = "Install Dependencies"
depends_on = ["clone"]
timeout_secs = 300
[steps.action]
type = "shell"
command = "npm"
args = ["install"]
[steps.action.env]
NODE_ENV = "production"
[steps.retry]
max_attempts = 3
delay_secs = 5
backoff = "exponential"
```

## Step Actions / 步骤动作

### Shell

Execute a shell command:

```toml
[steps.action]
type = "shell"
command = "npm"
args = ["install", "--production"]
[steps.action.env]
NODE_ENV = "production"
```

### PackageInstall

Install packages via a package manager:

```toml
[steps.action]
type = "packageInstall"
manager = "npm"
packages = ["express", "cors"]
```

Supported managers: `npm`, `pip`, `cargo`, `brew`, `apt`, `winget`

### EnvCheck

Assert an environment tool is available:

```toml
[steps.action]
type = "envCheck"
env_id = "node"
```

### Download *(not yet supported)*

```toml
[steps.action]
type = "download"
url = "https://example.com/file.tar.gz"
dest = "/tmp/file.tar.gz"
```

### Extract *(not yet supported)*

```toml
[steps.action]
type = "extract"
src = "/tmp/file.tar.gz"
dest = "/opt/project"
```

## Error Handling / 错误处理

Each step can configure error handling:

```toml
[[steps]]
id = "optional-step"
name = "Optional Step"
on_error = "skip"      # "fail" (default) | "skip" | "retry"
```

## Retry Configuration / 重试配置

```toml
[steps.retry]
max_attempts = 3       # Must be >= 1
delay_secs = 5         # Delay between retries (default: 3)
backoff = "fixed"      # "fixed" (default) | "exponential"
```

With exponential backoff, delays double: 5s → 10s → 20s

## Dependencies / 依赖关系

Steps can declare dependencies using `depends_on`:

```toml
[[steps]]
id = "build"
name = "Build"
depends_on = ["install"]  # Waits for "install" to complete
```

This creates a DAG (Directed Acyclic Graph). Steps without dependencies can execute in parallel.

## Variable Substitution / 变量替换

Use `${var_name}` syntax in shell commands:

```toml
[vars]
repo = "https://github.com/user/project.git"

[[steps]]
id = "clone"
name = "Clone"
[steps.action]
type = "shell"
command = "git"
args = ["clone", "${repo}"]
```

## Built-in Recipes / 内置配方

The application ships with two built-in recipes:

1. **OpenClaw Full** (`openclaw-full`) — Full OpenClaw deployment
2. **Claude Code** (`claude-code`) — Install Claude Code CLI globally
