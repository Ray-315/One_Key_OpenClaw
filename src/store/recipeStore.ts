import { create } from "zustand";
import type { Recipe } from "../ipc/types";
import { listRecipes, loadRecipeFile, deleteRecipe } from "../ipc/recipeApi";

interface RecipeState {
  recipes: Recipe[];
  loading: boolean;
  error: string | null;

  loadRecipes: () => Promise<void>;
  importFromFile: (path: string) => Promise<void>;
  remove: (recipeId: string) => Promise<void>;
}

export const useRecipeStore = create<RecipeState>((set) => ({
  recipes: [],
  loading: false,
  error: null,

  loadRecipes: async () => {
    set({ loading: true, error: null });
    try {
      const recipes = await listRecipes();
      set({ recipes, loading: false });
    } catch (e) {
      set({ error: String(e), loading: false });
    }
  },

  importFromFile: async (path) => {
    try {
      await loadRecipeFile(path);
      // Reload the full list after importing.
      const recipes = await listRecipes();
      set({ recipes });
    } catch (e) {
      set({ error: String(e) });
    }
  },

  remove: async (recipeId) => {
    try {
      await deleteRecipe(recipeId);
      const recipes = await listRecipes();
      set({ recipes });
    } catch (e) {
      set({ error: String(e) });
    }
  },
}));
