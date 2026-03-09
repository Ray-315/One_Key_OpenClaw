#[cfg(test)]
mod tests {
    use crate::task::step::{StepStatus, TaskStep};

    #[test]
    fn new_step_is_pending() {
        let step = TaskStep::new("s1", "Step 1", 3);
        assert!(matches!(step.status, StepStatus::Pending));
        assert_eq!(step.retry_count, 0);
        assert_eq!(step.max_retries, 3);
    }

    #[test]
    fn step_fields_set_correctly() {
        let step = TaskStep::new("install", "Install deps", 2);
        assert_eq!(step.id, "install");
        assert_eq!(step.name, "Install deps");
        assert_eq!(step.max_retries, 2);
        assert!(step.started_at.is_none());
        assert!(step.finished_at.is_none());
        assert!(step.exit_code.is_none());
        assert!(step.description.is_none());
    }

    #[test]
    fn step_status_serialization() {
        let status = StepStatus::Failed {
            error: "npm install failed".to_string(),
        };
        let json = serde_json::to_string(&status).unwrap();
        assert!(json.contains("failed"));
        assert!(json.contains("npm install failed"));
    }

    #[test]
    fn step_pending_eq() {
        assert_eq!(StepStatus::Pending, StepStatus::Pending);
        assert_ne!(StepStatus::Pending, StepStatus::Running);
    }
}
