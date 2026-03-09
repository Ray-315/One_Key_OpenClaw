import { useCallback } from "react";
import { useLogStore } from "../store/logStore";
import { useTauriEvent } from "./useTauriEvent";
import type { LogEntry } from "../ipc/types";

/**
 * Subscribe to the log://entry Tauri event and push entries to the log store.
 * Optionally filter by taskId.
 */
export function useLogStream(taskId?: string) {
  const addLog = useLogStore((s) => s.addLog);

  const handler = useCallback(
    (entry: LogEntry) => {
      if (!taskId || entry.taskId === taskId) {
        addLog(entry);
      }
    },
    [taskId, addLog]
  );

  useTauriEvent<LogEntry>("log://entry", handler);
}
