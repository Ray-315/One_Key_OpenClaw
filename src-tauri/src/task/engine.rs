#![allow(dead_code)]

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};
use tokio::sync::mpsc;

use crate::error::AppError;
use crate::log::pipeline::{LogEntry, LogLevel, LogPipeline, LogSource};
use crate::task::state_machine::{TaskControl, TaskEvent, TaskStateMachine};
use crate::task::step::{StepAction, StepStatus, TaskStep};

// ─── Task data types ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum TaskStatus {
    Idle,
    Running,
    Paused,
    Success,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Task {
    pub id: String,
    pub name: String,
    pub recipe_id: String,
    pub status: TaskStatus,
    pub steps: Vec<TaskStep>,
    pub created_at: u64,
    pub started_at: Option<u64>,
    pub finished_at: Option<u64>,
    /// Overall progress 0–100.
    pub progress: f32,
    pub error_summary: Option<String>,
}

// ─── Event payloads ─────────────────────────────────────────────────────────

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct TaskProgressEvent {
    task_id: String,
    progress: f32,
    current_step_id: Option<String>,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct TaskStatusEvent {
    task_id: String,
    status: TaskStatus,
}

// ─── Shared task handle ─────────────────────────────────────────────────────

/// A cloneable handle to a running task, used by the command layer.
#[derive(Clone)]
pub struct TaskHandle {
    pub task: Arc<Mutex<Task>>,
    pub control_tx: mpsc::Sender<TaskControl>,
}

// ─── TaskExecutor ────────────────────────────────────────────────────────────

pub struct TaskExecutor {
    app: AppHandle,
}

impl TaskExecutor {
    pub fn new(app: AppHandle) -> Self {
        Self { app }
    }

    /// Spawn a tokio task that executes all steps of `task` sequentially.
    /// Returns a [`TaskHandle`] that can be used to pause/resume/cancel.
    pub fn spawn(
        &self,
        task: Task,
    ) -> TaskHandle {
        let task = Arc::new(Mutex::new(task));
        let (control_tx, mut control_rx) = mpsc::channel::<TaskControl>(8);
        let app = self.app.clone();
        let task_clone = Arc::clone(&task);

        tokio::spawn(async move {
            // Transition to Running.
            {
                let mut t = task_clone.lock().unwrap();
                let _ = TaskStateMachine::transition(&mut t, TaskEvent::Start);
                t.started_at = Some(now_millis());
                let _ = app.emit(
                    "task://status",
                    TaskStatusEvent {
                        task_id: t.id.clone(),
                        status: t.status.clone(),
                    },
                );
            }

            let step_count = task_clone.lock().unwrap().steps.len();

            'steps: for step_index in 0..step_count {
                // Check for cancellation / pause before each step.
                loop {
                    // Drain control channel.
                    while let Ok(ctrl) = control_rx.try_recv() {
                        let mut t = task_clone.lock().unwrap();
                        match ctrl {
                            TaskControl::Cancel => {
                                let _ =
                                    TaskStateMachine::transition(&mut t, TaskEvent::Cancel);
                                TaskStateMachine::cancel_steps(&mut t);
                                t.finished_at = Some(now_millis());
                                let _ = app.emit(
                                    "task://status",
                                    TaskStatusEvent {
                                        task_id: t.id.clone(),
                                        status: t.status.clone(),
                                    },
                                );
                                break 'steps;
                            }
                            TaskControl::Pause => {
                                let _ =
                                    TaskStateMachine::transition(&mut t, TaskEvent::Pause);
                                let _ = app.emit(
                                    "task://status",
                                    TaskStatusEvent {
                                        task_id: t.id.clone(),
                                        status: t.status.clone(),
                                    },
                                );
                            }
                            TaskControl::Resume => {
                                let _ =
                                    TaskStateMachine::transition(&mut t, TaskEvent::Resume);
                                let _ = app.emit(
                                    "task://status",
                                    TaskStatusEvent {
                                        task_id: t.id.clone(),
                                        status: t.status.clone(),
                                    },
                                );
                            }
                        }
                    }

                    // If paused, wait before proceeding.
                    let paused = {
                        task_clone.lock().unwrap().status == TaskStatus::Paused
                    };
                    if paused {
                        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
                        continue;
                    }

                    // If cancelled, stop.
                    let cancelled = {
                        task_clone.lock().unwrap().status == TaskStatus::Cancelled
                    };
                    if cancelled {
                        break 'steps;
                    }

                    break; // Not paused, not cancelled — proceed with step.
                }

                // Mark step as Running.
                let (step_id, action) = {
                    let mut t = task_clone.lock().unwrap();
                    let step = &mut t.steps[step_index];
                    step.status = StepStatus::Running;
                    step.started_at = Some(now_millis());
                    let id = step.id.clone();
                    let action = step.action.clone();

                    // Emit step update.
                    let _ = app.emit("task://step-update", step.clone());
                    (id, action)
                };

                // Log start.
                LogPipeline::log_step(
                    &app,
                    {
                        let t = task_clone.lock().unwrap();
                        t.id.clone()
                    },
                    Some(step_id.clone()),
                    LogLevel::Info,
                    format!("▶ Starting step: {step_id}"),
                );

                // Execute the step.
                let result = execute_step(&app, &action, {
                    let t = task_clone.lock().unwrap();
                    t.id.clone()
                }, step_id.clone()).await;

                // Update step status.
                {
                    let mut t = task_clone.lock().unwrap();
                    let task_id_str = t.id.clone();
                    let step = &mut t.steps[step_index];
                    step.finished_at = Some(now_millis());

                    match result {
                        Ok(exit_code) => {
                            step.exit_code = Some(exit_code);
                            if exit_code == 0 {
                                step.status = StepStatus::Success;
                                LogPipeline::log_step(
                                    &app,
                                    task_id_str.clone(),
                                    Some(step_id.clone()),
                                    LogLevel::Info,
                                    format!("✔ Step '{step_id}' succeeded"),
                                );
                            } else {
                                let msg = format!("Step '{step_id}' exited with code {exit_code}");
                                step.status = StepStatus::Failed {
                                    error: msg.clone(),
                                };
                                LogPipeline::log_step(
                                    &app,
                                    task_id_str.clone(),
                                    Some(step_id.clone()),
                                    LogLevel::Error,
                                    format!("✖ {msg}"),
                                );
                            }
                        }
                        Err(e) => {
                            let msg = e.to_string();
                            step.status = StepStatus::Failed {
                                error: msg.clone(),
                            };
                            LogPipeline::log_step(
                                &app,
                                task_id_str.clone(),
                                Some(step_id.clone()),
                                LogLevel::Error,
                                format!("✖ Step '{step_id}' failed: {msg}"),
                            );
                        }
                    }

                    let _ = app.emit("task://step-update", step.clone());

                    // Recompute progress.
                    let done = t
                        .steps
                        .iter()
                        .filter(|s| {
                            matches!(
                                s.status,
                                StepStatus::Success
                                    | StepStatus::Failed { .. }
                                    | StepStatus::Skipped
                                    | StepStatus::Cancelled
                            )
                        })
                        .count();
                    t.progress = if step_count > 0 {
                        (done as f32 / step_count as f32) * 100.0
                    } else {
                        100.0
                    };

                    let _ = app.emit(
                        "task://progress",
                        TaskProgressEvent {
                            task_id: t.id.clone(),
                            progress: t.progress,
                            current_step_id: None,
                        },
                    );

                    // Stop on step failure.
                    if matches!(t.steps[step_index].status, StepStatus::Failed { .. }) {
                        t.error_summary = Some(format!("Step '{step_id}' failed"));
                        let _ = TaskStateMachine::transition(&mut t, TaskEvent::Fail);
                        t.finished_at = Some(now_millis());
                        let _ = app.emit(
                            "task://status",
                            TaskStatusEvent {
                                task_id: t.id.clone(),
                                status: t.status.clone(),
                            },
                        );
                        break 'steps;
                    }
                }
            }

            // Mark task as Success if not already Failed/Cancelled.
            {
                let mut t = task_clone.lock().unwrap();
                if t.status == TaskStatus::Running {
                    let _ = TaskStateMachine::transition(&mut t, TaskEvent::Complete);
                    t.progress = 100.0;
                    t.finished_at = Some(now_millis());
                    let _ = app.emit(
                        "task://status",
                        TaskStatusEvent {
                            task_id: t.id.clone(),
                            status: t.status.clone(),
                        },
                    );
                }
            }
        });

        TaskHandle { task, control_tx }
    }
}

// ─── Step executor ───────────────────────────────────────────────────────────

async fn execute_step(
    app: &AppHandle,
    action: &StepAction,
    task_id: String,
    step_id: String,
) -> Result<i32, AppError> {
    match action {
        StepAction::Shell { command, args, env } => {
            let mut cmd = tokio::process::Command::new(command);
            cmd.args(args);
            for (k, v) in env {
                cmd.env(k, v);
            }
            cmd.stdout(std::process::Stdio::piped());
            cmd.stderr(std::process::Stdio::piped());

            let mut child = cmd.spawn().map_err(|e| AppError::StepExecutionError {
                step_id: step_id.clone(),
                exit_code: None,
                stderr: e.to_string(),
            })?;

            // Stream stdout.
            let app_stdout = app.clone();
            let tid_stdout = task_id.clone();
            let sid_stdout = step_id.clone();
            if let Some(stdout) = child.stdout.take() {
                tokio::spawn(async move {
                    use tokio::io::AsyncBufReadExt;
                    let mut lines = tokio::io::BufReader::new(stdout).lines();
                    while let Ok(Some(line)) = lines.next_line().await {
                        LogPipeline::log_step(
                            &app_stdout,
                            tid_stdout.clone(),
                            Some(sid_stdout.clone()),
                            LogLevel::Info,
                            line,
                        );
                    }
                });
            }

            // Stream stderr.
            let app_stderr = app.clone();
            let tid_stderr = task_id.clone();
            let sid_stderr = step_id.clone();
            if let Some(stderr) = child.stderr.take() {
                tokio::spawn(async move {
                    use tokio::io::AsyncBufReadExt;
                    let mut lines = tokio::io::BufReader::new(stderr).lines();
                    while let Ok(Some(line)) = lines.next_line().await {
                        LogPipeline::log_step(
                            &app_stderr,
                            tid_stderr.clone(),
                            Some(sid_stderr.clone()),
                            LogLevel::Warn,
                            line,
                        );
                    }
                });
            }

            let status = child.wait().await.map_err(|e| AppError::StepExecutionError {
                step_id: step_id.clone(),
                exit_code: None,
                stderr: e.to_string(),
            })?;

            Ok(status.code().unwrap_or(-1))
        }

        StepAction::EnvCheck { env_id } => {
            // Re-use the env prober to verify the tool is available.
            use crate::env::prober::{EnvProber, EnvStatus};
            let item = EnvProber::probe(env_id);
            LogPipeline::log_step(
                app,
                task_id.clone(),
                Some(step_id.clone()),
                LogLevel::Info,
                format!("Env check '{}': {:?}", env_id, item.status),
            );
            match item.status {
                EnvStatus::Ok => Ok(0),
                EnvStatus::Missing => Err(AppError::StepExecutionError {
                    step_id: step_id.clone(),
                    exit_code: Some(1),
                    stderr: format!("Environment '{}' is not installed", env_id),
                }),
                other => Err(AppError::StepExecutionError {
                    step_id: step_id.clone(),
                    exit_code: Some(1),
                    stderr: format!("Environment '{}' check failed: {:?}", env_id, other),
                }),
            }
        }

        // PackageInstall, Download, Extract: system commands
        StepAction::PackageInstall { manager, packages } => {
            let args = match manager.as_str() {
                "npm" => {
                    let mut a = vec!["install".to_string(), "--production".to_string()];
                    a.extend_from_slice(packages);
                    a
                }
                "pip" | "pip3" => {
                    let mut a = vec!["install".to_string()];
                    a.extend_from_slice(packages);
                    a
                }
                "cargo" => {
                    let mut a = vec!["install".to_string()];
                    a.extend_from_slice(packages);
                    a
                }
                _ => {
                    let mut a = vec!["install".to_string()];
                    a.extend_from_slice(packages);
                    a
                }
            };
            let shell_action = StepAction::Shell {
                command: manager.clone(),
                args,
                env: HashMap::new(),
            };
            Box::pin(execute_step(app, &shell_action, task_id, step_id)).await
        }

        StepAction::Download { url, dest } => {
            // Use curl as a portable download mechanism.
            let shell_action = StepAction::Shell {
                command: "curl".to_string(),
                args: vec!["-L".to_string(), "-o".to_string(), dest.clone(), url.clone()],
                env: HashMap::new(),
            };
            Box::pin(execute_step(app, &shell_action, task_id, step_id)).await
        }

        StepAction::Extract { src, dest } => {
            let shell_action = StepAction::Shell {
                command: "tar".to_string(),
                args: vec!["-xzf".to_string(), src.clone(), "-C".to_string(), dest.clone()],
                env: HashMap::new(),
            };
            Box::pin(execute_step(app, &shell_action, task_id, step_id)).await
        }
    }
}

// ─── Helpers ─────────────────────────────────────────────────────────────────

fn now_millis() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

// Extend LogPipeline with a step-aware helper.
impl LogPipeline {
    pub fn log_step(
        app: &AppHandle,
        task_id: String,
        step_id: Option<String>,
        level: LogLevel,
        message: impl Into<String>,
    ) {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(1000);

        let entry = LogEntry {
            id: COUNTER.fetch_add(1, Ordering::Relaxed),
            task_id,
            step_id,
            level,
            message: message.into(),
            timestamp: now_millis(),
            source: LogSource::System,
        };
        let _ = app.emit("log://entry", &entry);
    }
}
