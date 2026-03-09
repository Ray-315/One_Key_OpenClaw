import { invoke } from "@tauri-apps/api/core";
import type { EnvItem } from "./types";

/** Probe all registered environment items */
export async function probeAllEnvs(): Promise<EnvItem[]> {
  return invoke<EnvItem[]>("probe_all_envs");
}

/** Probe a single environment item by id */
export async function probeEnv(envId: string): Promise<EnvItem> {
  return invoke<EnvItem>("probe_env", { id: envId });
}
