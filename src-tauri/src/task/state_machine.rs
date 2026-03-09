use serde::{Deserialize, Serialize};

use crate::task::engine::{Task, TaskStatus};
use crate::task::step::StepStatus;
use crate::error::AppError;

/// Control signals that can be sent to a running task.
#[derive(Debug, Clone, PartialEq)]
pub enum TaskControl {
    Pause,
    Resume,
    Cancel,
}

/// Events that drive task-level state transitions.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TaskEvent {
    Start,
    Pause,
    Resume,
    Complete,
    Fail,
    Cancel,
}

pub struct TaskStateMachine;

impl TaskStateMachine {
    /// Attempt a state transition and return the new status, or an error if
    /// the transition is not allowed from the current state.
    pub fn transition(task: &mut Task, event: TaskEvent) -> Result<(), AppError> {
        let new_status = match (&task.status, &event) {
            (TaskStatus::Idle, TaskEvent::Start) => TaskStatus::Running,
            (TaskStatus::Running, TaskEvent::Pause) => TaskStatus::Paused,
            (TaskStatus::Paused, TaskEvent::Resume) => TaskStatus::Running,
            (TaskStatus::Running, TaskEvent::Complete) => TaskStatus::Success,
            (TaskStatus::Running, TaskEvent::Fail) => TaskStatus::Failed,
            (TaskStatus::Idle, TaskEvent::Cancel)
            | (TaskStatus::Running, TaskEvent::Cancel)
            | (TaskStatus::Paused, TaskEvent::Cancel) => TaskStatus::Cancelled,
            _ => {
                return Err(AppError::InvalidStateTransition {
                    from: format!("{:?}", task.status),
                    event: format!("{:?}", event),
                })
            }
        };
        task.status = new_status;
        Ok(())
    }

    /// Mark all pending/waiting/running steps as cancelled.
    pub fn cancel_steps(task: &mut Task) {
        for step in task.steps.iter_mut() {
            match step.status {
                StepStatus::Pending
                | StepStatus::Waiting
                | StepStatus::Running => step.status = StepStatus::Cancelled,
                _ => {}
            }
        }
    }
}
