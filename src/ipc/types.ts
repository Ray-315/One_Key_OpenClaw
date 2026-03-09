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
