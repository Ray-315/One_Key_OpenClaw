import { create } from "zustand";
import type { LogEntry } from "../ipc/types";

const MAX_LOG_ENTRIES = 10000;

interface LogState {
  logs: LogEntry[];
  filterLevel: "all" | "debug" | "info" | "warn" | "error";

  addLog: (entry: LogEntry) => void;
  clear: () => void;
  setFilterLevel: (level: LogState["filterLevel"]) => void;
}

export const useLogStore = create<LogState>((set) => ({
  logs: [],
  filterLevel: "all",

  addLog: (entry) =>
    set((state) => {
      const logs = [...state.logs, entry];
      // Ring buffer: keep most recent entries
      if (logs.length > MAX_LOG_ENTRIES) {
        return { logs: logs.slice(logs.length - MAX_LOG_ENTRIES) };
      }
      return { logs };
    }),

  clear: () => set({ logs: [] }),

  setFilterLevel: (level) => set({ filterLevel: level }),
}));
