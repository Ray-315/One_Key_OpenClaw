import { create } from "zustand";
import type { Task, TaskStatus } from "../ipc/types";
import { getTask, listTasks } from "../ipc/taskApi";

interface TaskState {
  tasks: Record<string, Task>;
  activeTaskId: string | null;

  setActiveTask: (id: string | null) => void;
  upsertTask: (task: Task) => void;
  updateTaskStatus: (
    taskId: string,
    status: TaskStatus,
    errorSummary?: string
  ) => void;
  updateTaskProgress: (taskId: string, progress: number) => void;
  loadTasks: () => Promise<void>;
  refreshTask: (taskId: string) => Promise<void>;
}

export const useTaskStore = create<TaskState>((set, get) => ({
  tasks: {},
  activeTaskId: null,

  setActiveTask: (id) => set({ activeTaskId: id }),

  upsertTask: (task) =>
    set((state) => ({
      tasks: { ...state.tasks, [task.id]: task },
    })),

  updateTaskStatus: (taskId, status, errorSummary) =>
    set((state) => {
      const existing = state.tasks[taskId];
      if (!existing) return state;
      return {
        tasks: {
          ...state.tasks,
          [taskId]: { ...existing, status, errorSummary },
        },
      };
    }),

  updateTaskProgress: (taskId, progress) =>
    set((state) => {
      const existing = state.tasks[taskId];
      if (!existing) return state;
      return {
        tasks: { ...state.tasks, [taskId]: { ...existing, progress } },
      };
    }),

  loadTasks: async () => {
    try {
      const tasks = await listTasks();
      const map: Record<string, Task> = {};
      for (const t of tasks) map[t.id] = t;
      set({ tasks: map });
    } catch (e) {
      console.error("Failed to load tasks:", e);
    }
  },

  refreshTask: async (taskId) => {
    try {
      const task = await getTask(taskId);
      get().upsertTask(task);
    } catch (e) {
      console.error("Failed to refresh task:", e);
    }
  },
}));
