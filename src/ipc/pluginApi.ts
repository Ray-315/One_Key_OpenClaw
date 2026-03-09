import { invoke } from "@tauri-apps/api/core";
import type { PluginInfo } from "./types";

/** List all loaded plugins. */
export async function listPlugins(): Promise<PluginInfo[]> {
  return invoke<PluginInfo[]>("list_plugins");
}

/** Load a plugin from a directory containing plugin.toml. */
export async function loadPlugin(path: string): Promise<PluginInfo> {
  return invoke<PluginInfo>("load_plugin", { path });
}

/** Unload a plugin by its ID. */
export async function unloadPlugin(pluginId: string): Promise<void> {
  return invoke("unload_plugin", { pluginId });
}

/** Re-scan the plugins directory for new plugins. */
export async function scanPlugins(): Promise<number> {
  return invoke<number>("scan_plugins");
}
