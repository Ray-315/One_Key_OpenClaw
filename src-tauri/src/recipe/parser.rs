use crate::error::AppError;
use crate::recipe::schema::Recipe;

/// Parse a TOML string into a [`Recipe`].
pub fn parse_toml(content: &str) -> Result<Recipe, AppError> {
    toml::from_str(content).map_err(|e| AppError::RecipeParseError {
        path: "<inline>".into(),
        message: e.to_string(),
    })
}

/// Read a TOML file from disk and parse it into a [`Recipe`].
pub fn parse_file(path: &str) -> Result<Recipe, AppError> {
    let content = std::fs::read_to_string(path).map_err(|e| AppError::RecipeParseError {
        path: path.into(),
        message: format!("cannot read file: {e}"),
    })?;
    toml::from_str(&content).map_err(|e| AppError::RecipeParseError {
        path: path.into(),
        message: e.to_string(),
    })
}
