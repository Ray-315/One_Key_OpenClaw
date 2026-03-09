import { create } from "zustand";
import type { EnvItem } from "../ipc/types";
import { probeAllEnvs, probeEnv } from "../ipc/envApi";

interface EnvState {
  items: EnvItem[];
  loading: boolean;
  error: string | null;
  filter: "all" | "ok" | "missing" | "versionMismatch";

  setFilter: (filter: EnvState["filter"]) => void;
  probeAll: () => Promise<void>;
  probeSingle: (envId: string) => Promise<void>;
}

export const useEnvStore = create<EnvState>((set, get) => ({
  items: [],
  loading: false,
  error: null,
  filter: "all",

  setFilter: (filter) => set({ filter }),

  probeAll: async () => {
    set({ loading: true, error: null });
    try {
      const items = await probeAllEnvs();
      set({ items, loading: false });
    } catch (e) {
      set({ error: String(e), loading: false });
    }
  },

  probeSingle: async (envId: string) => {
    try {
      const item = await probeEnv(envId);
      const items = get().items;
      const idx = items.findIndex((i) => i.id === envId);
      if (idx >= 0) {
        const updated = [...items];
        updated[idx] = item;
        set({ items: updated });
      } else {
        set({ items: [...items, item] });
      }
    } catch (e) {
      set({ error: String(e) });
    }
  },
}));
