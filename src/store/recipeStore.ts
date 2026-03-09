import { create } from "zustand";
import type { Recipe } from "../ipc/types";
import { listRecipes } from "../ipc/recipeApi";

interface RecipeState {
  recipes: Recipe[];
  loading: boolean;
  error: string | null;

  loadRecipes: () => Promise<void>;
  upsertRecipe: (recipe: Recipe) => void;
  removeRecipe: (id: string) => void;
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

  upsertRecipe: (recipe) =>
    set((state) => {
      const idx = state.recipes.findIndex((r) => r.id === recipe.id);
      if (idx >= 0) {
        const updated = [...state.recipes];
        updated[idx] = recipe;
        return { recipes: updated };
      }
      return { recipes: [...state.recipes, recipe] };
    }),

  removeRecipe: (id) =>
    set((state) => ({
      recipes: state.recipes.filter((r) => r.id !== id),
    })),
}));
