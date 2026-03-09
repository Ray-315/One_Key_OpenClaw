import { invoke } from "@tauri-apps/api/core";
import type { Recipe, ValidationIssue } from "./types";

/** List all registered recipes. */
export async function listRecipes(): Promise<Recipe[]> {
  return invoke<Recipe[]>("list_recipes");
}

/** Load a recipe from a TOML file path and register it. */
export async function loadRecipeFile(path: string): Promise<Recipe> {
  return invoke<Recipe>("load_recipe_file", { path });
}

/** Validate a recipe and return any issues. */
export async function validateRecipe(
  recipe: Recipe
): Promise<ValidationIssue[]> {
  return invoke<ValidationIssue[]>("validate_recipe_cmd", { recipe });
}

/** Save (add or replace) a recipe in the registry. */
export async function saveRecipe(recipe: Recipe): Promise<void> {
  return invoke("save_recipe", { recipe });
}

/** Delete a recipe from the registry by ID. */
export async function deleteRecipe(recipeId: string): Promise<void> {
  return invoke("delete_recipe", { recipeId });
}

/** Fetch a recipe from a remote URL and register it. */
export async function fetchRecipeUrl(url: string): Promise<Recipe> {
  return invoke<Recipe>("fetch_recipe_url", { url });
}
