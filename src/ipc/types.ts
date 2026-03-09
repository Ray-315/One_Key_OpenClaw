// TypeScript types matching Rust backend structures

export type EnvStatus =
  | { type: "ok" }
  | { type: "missing" }
  | { type: "versionMismatch"; found: string; required: string }
  | { type: "error"; message: string }
  | { type: "checking" };

export interface InstallHint {
  macos?: string;
  windows?: string;
  linux?: string;
}

export interface EnvItem {
  id: string;
  name: string;
  status: EnvStatus;
  version?: string;
  requiredVersion?: string;
  path?: string;
  installHint?: InstallHint;
  checkedAt: number;
}

export interface LogEntry {
  id: number;
  taskId: string;
  stepId?: string;
  level: "debug" | "info" | "warn" | "error";
  message: string;
  timestamp: number;
  source: "stdout" | "stderr" | "system" | { pluginId: string };
}

// ─── Task types ──────────────────────────────────────────────────────────────

export type StepStatus =
  | { type: "pending" }
  | { type: "waiting" }
  | { type: "running" }
  | { type: "success" }
  | { type: "failed"; error: string }
  | { type: "skipped" }
  | { type: "cancelled" };

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
  | "idle"
  | "running"
  | "paused"
  | "success"
  | "failed"
  | "cancelled";

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

// ─── Recipe types ─────────────────────────────────────────────────────────────

export interface EnvRequirement {
  envId: string;
  version?: string;
  optional: boolean;
}

export interface RecipeMetadata {
  createdAt?: string;
  sourceUrl?: string;
  checksum?: string;
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
  metadata: RecipeMetadata;
}

export interface RecipeStep {
  id: string;
  name: string;
  description?: string;
  action: RecipeStepAction;
  dependsOn: string[];
  condition?: string;
  timeoutSecs?: number;
  onError: "fail" | "skip" | "retry";
}

export type RecipeStepAction =
  | { type: "shell"; command: string; args: string[]; env: Record<string, string> }
  | { type: "packageInstall"; manager: string; packages: string[] }
  | { type: "envCheck"; envId: string }
  | { type: "download"; url: string; dest: string }
  | { type: "extract"; src: string; dest: string };

export interface ValidationIssue {
  field: string;
  message: string;
  severity: "Error" | "Warning";
}

// ─── Event payloads ──────────────────────────────────────────────────────────

export interface TaskProgressEvent {
  taskId: string;
  progress: number;
  currentStepId?: string;
}

export interface TaskStatusEvent {
  taskId: string;
  status: TaskStatus;
}

