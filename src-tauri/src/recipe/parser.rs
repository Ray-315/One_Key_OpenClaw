use crate::error::AppError;
use crate::recipe::schema::{validate_recipe, IssueSeverity, Recipe};

fn validate_parsed_recipe(recipe: Recipe, path: &str) -> Result<Recipe, AppError> {
    let errors: Vec<String> = validate_recipe(&recipe)
        .into_iter()
        .filter(|issue| matches!(issue.severity, IssueSeverity::Error))
        .map(|issue| format!("{}: {}", issue.field, issue.message))
        .collect();

    if errors.is_empty() {
        Ok(recipe)
    } else {
        Err(AppError::RecipeParseError {
            path: path.into(),
            message: errors.join("\n"),
        })
    }
}

/// Parse a TOML string into a [`Recipe`].
pub fn parse_toml(content: &str) -> Result<Recipe, AppError> {
    let recipe = toml::from_str(content).map_err(|e| AppError::RecipeParseError {
        path: "<inline>".into(),
        message: e.to_string(),
    })?;
    validate_parsed_recipe(recipe, "<inline>")
}

/// Read a TOML file from disk and parse it into a [`Recipe`].
pub fn parse_file(path: &str) -> Result<Recipe, AppError> {
    let content = std::fs::read_to_string(path).map_err(|e| AppError::RecipeParseError {
        path: path.into(),
        message: format!("cannot read file: {e}"),
    })?;
    let recipe = toml::from_str(&content).map_err(|e| AppError::RecipeParseError {
        path: path.into(),
        message: e.to_string(),
    })?;
    validate_parsed_recipe(recipe, path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_toml_rejects_invalid_recipe_after_deserialization() {
        let result = parse_toml(
            r#"
version = "1"
id = "demo"
name = "Demo"

[[steps]]
id = "download"
name = "Download"
[steps.action]
type = "download"
url = "https://example.com/file.zip"
dest = "/tmp/file.zip"
"#,
        );

        match result {
            Err(AppError::RecipeParseError { message, .. }) => {
                assert!(message.contains("download steps are not supported yet"));
            }
            other => panic!("expected validation parse error, got {other:?}"),
        }
    }
}
