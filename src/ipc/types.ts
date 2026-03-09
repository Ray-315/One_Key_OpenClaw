// TypeScript types matching Rust backend structures

// ---------------------------------------------------------------------------
// Environment types
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// Log types
// ---------------------------------------------------------------------------

export interface LogEntry {
  id: number;
  taskId: string;
  stepId?: string;
  level: "debug" | "info" | "warn" | "error";
  message: string;
  timestamp: number;
  source: "stdout" | "stderr" | "system" | { pluginId: string };
}

// ---------------------------------------------------------------------------
// Task / Step types
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// Recipe types
// ---------------------------------------------------------------------------

export interface EnvRequirement {
  envId: string;
  version?: string;
  optional: boolean;
}

export type StepAction =
  | {
      type: "shell";
      command: string;
      args: string[];
      env: Record<string, string>;
    }
  | { type: "packageInstall"; manager: string; packages: string[] }
  | { type: "envCheck"; envId: string }
  | { type: "download"; url: string; dest: string }
  | { type: "extract"; src: string; dest: string };

export interface RetryConfig {
  maxAttempts: number;
  delaySecs: number;
  backoff: "fixed" | "exponential";
}

export type OnErrorStrategy = "fail" | "skip" | "retry";

export interface RecipeStep {
  id: string;
  name: string;
  description?: string;
  action: StepAction;
  dependsOn: string[];
  condition?: string;
  retry?: RetryConfig;
  timeoutSecs?: number;
  onError: OnErrorStrategy;
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

export interface ValidationIssue {
  field: string;
  message: string;
  severity: "error" | "warning";
}

// ---------------------------------------------------------------------------
// Error diagnostic types (Phase 3)
// ---------------------------------------------------------------------------

export type ErrorCategory =
  | "networkError"
  | "permissionDenied"
  | "missingDependency"
  | "versionConflict"
  | "diskSpace"
  | "timeout"
  | "unknown";

export type FixAction =
  | { type: "runCommand"; command: string; args: string[] }
  | { type: "retryStep"; stepId: string }
  | { type: "installEnv"; envId: string }
  | { type: "openUrl"; url: string };

export interface FixSuggestion {
  title: string;
  description: string;
  action?: FixAction;
}

export interface ErrorRule {
  id: string;
  pattern: string;
  description: string;
  category: ErrorCategory;
  suggestions: FixSuggestion[];
}

export interface DiagnosticReport {
  taskId: string;
  stepId: string;
  rawError: string;
  matchedRule?: ErrorRule;
  suggestions: FixSuggestion[];
  autoFixable: boolean;
}

// ---------------------------------------------------------------------------
// DAG graph types (Phase 3)
// ---------------------------------------------------------------------------

export interface TaskGraphNode {
  id: string;
  name: string;
  description?: string;
  layer: number;
  dependsOn: string[];
}

export interface TaskGraphEdge {
  id: string;
  source: string;
  target: string;
}

export interface TaskGraphData {
  nodes: TaskGraphNode[];
  edges: TaskGraphEdge[];
}

// ---------------------------------------------------------------------------
// Event payloads (backend → frontend push)
// ---------------------------------------------------------------------------

export interface TaskProgressEvent {
  taskId: string;
  progress: number;
  currentStepId?: string;
}

export interface TaskStatusEvent {
  taskId: string;
  status: TaskStatus;
  errorSummary?: string;
}
