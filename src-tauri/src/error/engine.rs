use regex::Regex;
use serde::Deserialize;

use crate::error::{DiagnosticReport, ErrorRule, FixSuggestion};

// Built-in rules embedded at compile-time.
const BUILTIN_RULES_TOML: &str = include_str!("../../assets/error_rules.toml");

#[derive(Debug, Deserialize)]
struct RulesFile {
    #[serde(default)]
    rules: Vec<ErrorRule>,
}

/// Matches error text against a library of rules and produces a
/// [`DiagnosticReport`] with human-readable fix suggestions.
pub struct ErrorDiagnosticEngine {
    rules: Vec<ErrorRule>,
}

impl ErrorDiagnosticEngine {
    /// Create an engine pre-loaded with the built-in error rules.
    pub fn with_builtins() -> Self {
        let rules_file: RulesFile =
            toml::from_str(BUILTIN_RULES_TOML).unwrap_or_else(|e| {
                eprintln!("[ErrorDiagnosticEngine] Failed to parse built-in rules: {e}");
                RulesFile { rules: Vec::new() }
            });
        Self {
            rules: rules_file.rules,
        }
    }

    /// Diagnose a raw error string for a specific task/step.
    pub fn diagnose(&self, task_id: &str, step_id: &str, raw_error: &str) -> DiagnosticReport {
        let matched_rule = self.rules.iter().find(|rule| {
            Regex::new(&rule.pattern)
                .map(|re| re.is_match(raw_error))
                .unwrap_or(false)
        });

        let mut suggestions: Vec<FixSuggestion> = matched_rule
            .map(|r| r.suggestions.clone())
            .unwrap_or_default();

        // Always add a generic "retry step" suggestion as a fallback.
        if suggestions.is_empty() {
            suggestions.push(FixSuggestion {
                title: "重试该步骤".into(),
                description: "有时临时性错误在重试后即可解决。".into(),
                action: Some(crate::error::FixAction::RetryStep {
                    step_id: step_id.to_string(),
                }),
            });
        }

        let auto_fixable = suggestions.iter().any(|s| s.action.is_some());

        DiagnosticReport {
            task_id: task_id.to_string(),
            step_id: step_id.to_string(),
            raw_error: raw_error.to_string(),
            matched_rule: matched_rule.cloned(),
            suggestions,
            auto_fixable,
        }
    }
}
