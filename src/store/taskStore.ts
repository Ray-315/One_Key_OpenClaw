import { create } from "zustand";
import type { Task, TaskStep, TaskStatusEvent, TaskProgressEvent } from "../ipc/types";
import {
  startTask,
  pauseTask,
  resumeTask,
  cancelTask,
  getTask,
  listTasks,
} from "../ipc/taskApi";

interface TaskState {
  tasks: Record<string, Task>;
  activeTaskId: string | null;
  loading: boolean;
  error: string | null;

  /** Load all tasks from backend */
  loadTasks: () => Promise<void>;
  /** Start a task from a recipe */
  start: (recipeId: string, vars?: Record<string, string>) => Promise<string>;
  /** Pause the active task */
  pause: () => Promise<void>;
  /** Resume the active task */
  resume: () => Promise<void>;
  /** Cancel the active task */
  cancel: () => Promise<void>;
  /** Set the active task id */
  setActiveTask: (taskId: string | null) => void;
  /** Apply a status/progress event from Tauri */
  applyStatusEvent: (event: TaskStatusEvent) => void;
  applyProgressEvent: (event: TaskProgressEvent) => void;
  applyStepUpdate: (step: TaskStep & { taskId?: string }) => void;
}

export const useTaskStore = create<TaskState>((set, get) => ({
  tasks: {},
  activeTaskId: null,
  loading: false,
  error: null,

  loadTasks: async () => {
    set({ loading: true, error: null });
    try {
      const list = await listTasks();
      const tasks: Record<string, Task> = {};
      for (const t of list) tasks[t.id] = t;
      set({ tasks, loading: false });
    } catch (e) {
      set({ error: String(e), loading: false });
    }
  },

  start: async (recipeId, vars = {}) => {
    set({ loading: true, error: null });
    try {
      const taskId = await startTask(recipeId, vars);
      // Fetch the initial task snapshot.
      const task = await getTask(taskId);
      set((s) => ({
        tasks: { ...s.tasks, [taskId]: task },
        activeTaskId: taskId,
        loading: false,
      }));
      return taskId;
    } catch (e) {
      set({ error: String(e), loading: false });
      throw e;
    }
  },

  pause: async () => {
    const { activeTaskId } = get();
    if (!activeTaskId) return;
    try {
      await pauseTask(activeTaskId);
    } catch (e) {
      set({ error: String(e) });
    }
  },

  resume: async () => {
    const { activeTaskId } = get();
    if (!activeTaskId) return;
    try {
      await resumeTask(activeTaskId);
    } catch (e) {
      set({ error: String(e) });
    }
  },

  cancel: async () => {
    const { activeTaskId } = get();
    if (!activeTaskId) return;
    try {
      await cancelTask(activeTaskId);
    } catch (e) {
      set({ error: String(e) });
    }
  },

  setActiveTask: (taskId) => set({ activeTaskId: taskId }),

  applyStatusEvent: ({ taskId, status }) => {
    set((s) => {
      const task = s.tasks[taskId];
      if (!task) return s;
      return {
        tasks: { ...s.tasks, [taskId]: { ...task, status } },
      };
    });
  },

  applyProgressEvent: ({ taskId, progress }) => {
    set((s) => {
      const task = s.tasks[taskId];
      if (!task) return s;
      return {
        tasks: { ...s.tasks, [taskId]: { ...task, progress } },
      };
    });
  },

  applyStepUpdate: (updatedStep) => {
    set((s) => {
      // The step update payload mirrors TaskStep; taskId is inferred from activeTaskId.
      const taskId = get().activeTaskId;
      if (!taskId) return s;
      const task = s.tasks[taskId];
      if (!task) return s;
      const steps = task.steps.map((st) =>
        st.id === updatedStep.id ? { ...st, ...updatedStep } : st
      );
      return {
        tasks: { ...s.tasks, [taskId]: { ...task, steps } },
      };
    });
  },
}));
