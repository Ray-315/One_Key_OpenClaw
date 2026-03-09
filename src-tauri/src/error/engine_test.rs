#[cfg(test)]
mod tests {
    use crate::error::engine::ErrorDiagnosticEngine;

    #[test]
    fn diagnose_returns_report_with_fallback() {
        let engine = ErrorDiagnosticEngine::with_builtins();
        let report = engine.diagnose("t1", "s1", "some random error that matches nothing");
        assert_eq!(report.task_id, "t1");
        assert_eq!(report.step_id, "s1");
        assert!(
            !report.suggestions.is_empty(),
            "should have fallback suggestion"
        );
    }

    #[test]
    fn diagnose_matches_npm_eacces() {
        let engine = ErrorDiagnosticEngine::with_builtins();
        let report = engine.diagnose(
            "task-1",
            "install",
            "npm ERR! Error: EACCES: permission denied, mkdir '/usr/local/lib'",
        );
        assert!(
            report.matched_rule.is_some(),
            "should match npm_eacces rule"
        );
        let rule = report.matched_rule.unwrap();
        assert_eq!(rule.id, "npm_eacces");
    }

    #[test]
    fn diagnose_matches_network_timeout() {
        let engine = ErrorDiagnosticEngine::with_builtins();
        let report = engine.diagnose(
            "task-1",
            "download",
            "error: ETIMEDOUT connecting to registry.npmjs.org",
        );
        assert!(report.matched_rule.is_some());
        let rule = report.matched_rule.unwrap();
        assert_eq!(rule.id, "network_timeout");
    }

    #[test]
    fn diagnose_matches_node_missing() {
        let engine = ErrorDiagnosticEngine::with_builtins();
        let report = engine.diagnose("task-1", "build", "sh: node: command not found");
        assert!(report.matched_rule.is_some());
        let rule = report.matched_rule.unwrap();
        assert_eq!(rule.id, "node_missing");
    }

    #[test]
    fn diagnose_auto_fixable_flag() {
        let engine = ErrorDiagnosticEngine::with_builtins();
        // Matched rule with actions should be auto-fixable
        let report = engine.diagnose("t1", "s1", "EACCES: permission denied");
        assert!(report.auto_fixable);
    }

    #[test]
    fn diagnose_unmatched_has_retry_suggestion() {
        let engine = ErrorDiagnosticEngine::with_builtins();
        let report = engine.diagnose("t1", "s1", "completely unknown error xyz");
        assert_eq!(report.suggestions.len(), 1);
        assert!(report.suggestions[0].title.contains("重试"));
    }
}
