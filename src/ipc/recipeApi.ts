import { invoke } from "@tauri-apps/api/core";
import type { Recipe, ValidationIssue } from "./types";

/** List all registered recipes */
export async function listRecipes(): Promise<Recipe[]> {
  return invoke<Recipe[]>("list_recipes");
}

/** Load a recipe from a TOML file path */
export async function loadRecipeFile(path: string): Promise<Recipe> {
  return invoke<Recipe>("load_recipe_file", { path });
}

/** Validate a recipe and return any issues */
export async function validateRecipe(
  recipe: Recipe
): Promise<ValidationIssue[]> {
  return invoke<ValidationIssue[]>("validate_recipe", { recipe });
}

/** Save (upsert) a recipe */
export async function saveRecipe(recipe: Recipe): Promise<void> {
  return invoke<void>("save_recipe", { recipe });
}

/** Delete a recipe by id */
export async function deleteRecipe(recipeId: string): Promise<void> {
  return invoke<void>("delete_recipe", { recipeId });
}

/** Get a single recipe by id */
export async function getRecipe(recipeId: string): Promise<Recipe | null> {
  return invoke<Recipe | null>("get_recipe", { recipeId });
}
