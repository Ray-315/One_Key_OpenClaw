use std::collections::{HashMap, HashSet};
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};
use tokio::sync::mpsc;
use tokio::task::JoinSet;

use crate::env::prober::{EnvProber, EnvStatus};
use crate::log::pipeline::{LogLevel, LogPipeline, LogSource};
use crate::recipe::schema::{
    BackoffStrategy, OnErrorStrategy, PackageManager, Recipe, RecipeStep, StepAction,
};
use crate::task::graph::TaskGraph;
use crate::task::state_machine::TaskControl;
use crate::task::step::{StepStatus, TaskStep};

// ---------------------------------------------------------------------------
// Task
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum TaskStatus {
    Idle,
    Running,
    Paused,
    Success,
    Failed,
    Cancelled,
}

/// Runtime task created from a recipe.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Task {
    pub id: String,
    pub name: String,
    pub recipe_id: String,
    pub status: TaskStatus,
    /// Frontend-visible step statuses (matches order of recipe steps).
    pub steps: Vec<TaskStep>,
    pub created_at: u64,
    pub started_at: Option<u64>,
    pub finished_at: Option<u64>,
    /// Overall progress 0–100.
    pub progress: f32,
    pub error_summary: Option<String>,
}

// ---------------------------------------------------------------------------
// Event payloads emitted to the frontend
// ---------------------------------------------------------------------------

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskProgressEvent {
    pub task_id: String,
    pub progress: f32,
    pub current_step_id: Option<String>,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskStatusEvent {
    pub task_id: String,
    pub status: TaskStatus,
    pub error_summary: Option<String>,
}

// ---------------------------------------------------------------------------
// Helper: current time in milliseconds
// ---------------------------------------------------------------------------

fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

// ---------------------------------------------------------------------------
// Variable substitution
// ---------------------------------------------------------------------------

/// Replace `${VAR}` placeholders in `s` using `vars`, then expand common
/// environment variables such as `${HOME}`.
fn substitute(s: &str, vars: &HashMap<String, String>) -> String {
    let mut out = s.to_string();
    for (k, v) in vars {
        out = out.replace(&format!("${{{k}}}"), v);
    }
    // Expand a small set of OS env vars.
    for name in &["HOME", "USERPROFILE", "APPDATA", "TEMP", "TMP", "PATH"] {
        if let Ok(val) = std::env::var(name) {
            out = out.replace(&format!("${{{name}}}"), &val);
        }
    }
    out
}

/// Apply substitution to all string fields within a `RecipeStep` action.
fn substitute_step(step: &RecipeStep, vars: &HashMap<String, String>) -> RecipeStep {
    let action = match &step.action {
        StepAction::Shell { command, args, env } => StepAction::Shell {
            command: substitute(command, vars),
            args: args.iter().map(|a| substitute(a, vars)).collect(),
            env: env
                .iter()
                .map(|(k, v)| (k.clone(), substitute(v, vars)))
                .collect(),
        },
        StepAction::PackageInstall { manager, packages } => StepAction::PackageInstall {
            manager: manager.clone(),
            packages: packages.iter().map(|p| substitute(p, vars)).collect(),
        },
        StepAction::EnvCheck { env_id } => StepAction::EnvCheck {
            env_id: substitute(env_id, vars),
        },
        StepAction::Download { url, dest } => StepAction::Download {
            url: substitute(url, vars),
            dest: dest.clone(),
        },
        StepAction::Extract { src, dest } => StepAction::Extract {
            src: src.clone(),
            dest: dest.clone(),
        },
    };
    RecipeStep { action, ..step.clone() }
}

// ---------------------------------------------------------------------------
// Single-step execution
// ---------------------------------------------------------------------------

/// Execute one step action and return `(exit_code, error_message)`.
/// Stdout / stderr lines are emitted as log entries.
fn execute_action(
    action: &StepAction,
    task_id: &str,
    step_id: &str,
    app: &AppHandle,
) -> (Option<i32>, Option<String>) {
    match action {
        // -------------------------------------------------------------------
        StepAction::Shell { command, args, env } => {
            let mut cmd = Command::new(command);
            cmd.args(args)
                .envs(env)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped());

            match cmd.spawn() {
                Err(e) => (
                    None,
                    Some(format!("Failed to spawn '{command}': {e}")),
                ),
                Ok(mut child) => {
                    // Stream stdout
                    if let Some(stdout) = child.stdout.take() {
                        let app2 = app.clone();
                        let tid = task_id.to_string();
                        let sid = step_id.to_string();
                        std::thread::spawn(move || {
                            for line in BufReader::new(stdout).lines().map_while(Result::ok) {
                                LogPipeline::log_step(
                                    &app2, LogLevel::Info, &tid, &sid, LogSource::Stdout, &line,
                                );
                            }
                        });
                    }
                    // Stream stderr
                    if let Some(stderr) = child.stderr.take() {
                        let app2 = app.clone();
                        let tid = task_id.to_string();
                        let sid = step_id.to_string();
                        std::thread::spawn(move || {
                            for line in BufReader::new(stderr).lines().map_while(Result::ok) {
                                LogPipeline::log_step(
                                    &app2, LogLevel::Warn, &tid, &sid, LogSource::Stderr, &line,
                                );
                            }
                        });
                    }

                    match child.wait() {
                        Err(e) => (None, Some(format!("Failed to wait on process: {e}"))),
                        Ok(status) => {
                            let code = status.code();
                            if status.success() {
                                (code, None)
                            } else {
                                (
                                    code,
                                    Some(format!(
                                        "Command exited with status {}",
                                        code.map(|c| c.to_string())
                                            .unwrap_or_else(|| "unknown".into())
                                    )),
                                )
                            }
                        }
                    }
                }
            }
        }

        // -------------------------------------------------------------------
        StepAction::PackageInstall { manager, packages } => {
            let (cmd, args) = package_install_cmd(manager, packages);
            let action = StepAction::Shell {
                command: cmd,
                args,
                env: HashMap::new(),
            };
            execute_action(&action, task_id, step_id, app)
        }

        // -------------------------------------------------------------------
        StepAction::EnvCheck { env_id } => {
            let item = EnvProber::probe(env_id);
            LogPipeline::log_step(
                app,
                LogLevel::Info,
                task_id,
                step_id,
                LogSource::System,
                &format!("env check: {} → {:?}", env_id, item.status),
            );
            match &item.status {
                EnvStatus::Ok => (Some(0), None),
                EnvStatus::Missing => (
                    Some(1),
                    Some(format!("Environment '{}' is not installed", env_id)),
                ),
                EnvStatus::VersionMismatch { found, required } => (
                    Some(1),
                    Some(format!(
                        "Environment '{}' version mismatch: found {}, required {}",
                        env_id, found, required
                    )),
                ),
                EnvStatus::Error { message } => {
                    (Some(1), Some(format!("EnvCheck error: {}", message)))
                }
                EnvStatus::Checking => (None, Some("EnvCheck left in Checking state".into())),
            }
        }

        // -------------------------------------------------------------------
        StepAction::Download { url, dest } => {
            (Some(1), Some(format!("Download not yet supported: {url} → {dest:?}")))
        }
        StepAction::Extract { src, dest } => {
            (Some(1), Some(format!("Extract not yet supported: {src:?} → {dest:?}")))
        }
    }
}

/// Build the CLI invocation for a package-manager install command.
fn package_install_cmd(manager: &PackageManager, packages: &[String]) -> (String, Vec<String>) {
    let mut args: Vec<String> = Vec::new();
    let cmd = match manager {
        PackageManager::Npm => {
            args.push("install".into());
            args.push("--global".into());
            "npm".into()
        }
        PackageManager::Pip => {
            args.push("install".into());
            "pip".into()
        }
        PackageManager::Cargo => {
            args.push("install".into());
            "cargo".into()
        }
        PackageManager::Brew => {
            args.push("install".into());
            "brew".into()
        }
        PackageManager::Apt => {
            args.push("install".into());
            args.push("-y".into());
            "apt-get".into()
        }
        PackageManager::Winget => {
            args.push("install".into());
            "winget".into()
        }
    };
    args.extend(packages.iter().cloned());
    (cmd, args)
}

// ---------------------------------------------------------------------------
// Task executor (serial, Phase 2)
// ---------------------------------------------------------------------------

/// Update a `TaskStep` inside a `Task` stored in the shared state and emit an
/// event to the frontend.
fn update_step(
    app: &AppHandle,
    task_arc: &Arc<Mutex<Task>>,
    step_id: &str,
    f: impl FnOnce(&mut TaskStep),
) {
    let step_snapshot = {
        let mut task = task_arc.lock().unwrap();
        if let Some(step) = task.steps.iter_mut().find(|s| s.id == step_id) {
            f(step);
            Some(step.clone())
        } else {
            None
        }
    };
    if let Some(step) = step_snapshot {
        let _ = app.emit("task://step-update", &step);
    }
}

/// Recalculate task progress and emit a progress event.
fn emit_progress(app: &AppHandle, task_arc: &Arc<Mutex<Task>>, current_step_id: Option<&str>) {
    let (task_id, progress) = {
        let mut task = task_arc.lock().unwrap();
        let total = task.steps.len();
        let done = task
            .steps
            .iter()
            .filter(|s| {
                matches!(
                    s.status,
                    StepStatus::Success | StepStatus::Skipped | StepStatus::Failed { .. }
                )
            })
            .count();
        let p = if total == 0 {
            100.0
        } else {
            (done as f32 / total as f32) * 100.0
        };
        task.progress = p;
        (task.id.clone(), p)
    };
    let _ = app.emit(
        "task://progress",
        TaskProgressEvent {
            task_id,
            progress,
            current_step_id: current_step_id.map(str::to_string),
        },
    );
}

/// Update the task status and emit a status event.
fn set_task_status(
    app: &AppHandle,
    task_arc: &Arc<Mutex<Task>>,
    status: TaskStatus,
    error_summary: Option<String>,
) {
    let (task_id, status_clone, err_clone) = {
        let mut task = task_arc.lock().unwrap();
        task.status = status.clone();
        task.error_summary = error_summary.clone();
        match &status {
            TaskStatus::Running if task.started_at.is_none() => {
                task.started_at = Some(now_ms());
            }
            TaskStatus::Success
            | TaskStatus::Failed
            | TaskStatus::Cancelled => {
                task.finished_at = Some(now_ms());
            }
            _ => {}
        }
        (task.id.clone(), status, error_summary)
    };
    let _ = app.emit(
        "task://status",
        TaskStatusEvent {
            task_id,
            status: status_clone,
            error_summary: err_clone,
        },
    );
}

/// Main async executor. Runs all steps serially and handles pause / cancel.
/// Outcome reported by a single step execution task.
enum StepOutcome {
    /// Step finished successfully.
    Success { step_id: String },
    /// Step failed but `on_error = skip`, so the task may continue.
    Skipped { step_id: String },
    /// Step failed and the task should abort.
    Failed { step_id: String, error: String },
}

/// Async wrapper: execute a single step (with retry logic) and report the
/// outcome.  This function is spawned as a tokio task so it can run in
/// parallel with other steps.
async fn run_step(
    app: AppHandle,
    task_arc: Arc<Mutex<Task>>,
    task_id: String,
    step: RecipeStep,
) -> StepOutcome {
    let step_id = step.id.clone();
    let max_attempts = step.retry.as_ref().map(|r| r.max_attempts).unwrap_or(0);
    let should_retry = step.on_error == OnErrorStrategy::Retry || step.retry.is_some();

    // Mark step Running.
    update_step(&app, &task_arc, &step_id, |s| {
        s.status = StepStatus::Running;
        s.started_at = Some(now_ms());
    });
    emit_progress(&app, &task_arc, Some(&step_id));

    LogPipeline::log_step(
        &app,
        LogLevel::Info,
        &task_id,
        &step_id,
        LogSource::System,
        &format!("Starting step: {}", step.name),
    );

    let mut attempt = 0u8;

    loop {
        // Run the blocking execute_action on a thread-pool thread.
        let action = step.action.clone();
        let task_id_c = task_id.clone();
        let step_id_c = step_id.clone();
        let app_c = app.clone();

        let (exit_code, error) =
            tokio::task::spawn_blocking(move || execute_action(&action, &task_id_c, &step_id_c, &app_c))
                .await
                .unwrap_or_else(|_| (None, Some("step task panicked".into())));

        if error.is_none() {
            // Success
            update_step(&app, &task_arc, &step_id, |s| {
                s.status = StepStatus::Success;
                s.finished_at = Some(now_ms());
                s.exit_code = exit_code;
                s.retry_count = attempt;
            });
            emit_progress(&app, &task_arc, None);
            return StepOutcome::Success { step_id };
        }

        let err_msg = error.unwrap();

        if attempt < max_attempts && should_retry {
            attempt += 1;
            let delay = {
                let cfg = step.retry.as_ref();
                let base = cfg.map(|r| r.delay_secs).unwrap_or(3);
                if cfg
                    .map(|r| r.backoff == BackoffStrategy::Exponential)
                    .unwrap_or(false)
                {
                    base * 2u64.pow(attempt as u32 - 1)
                } else {
                    base
                }
            };

            update_step(&app, &task_arc, &step_id, |s| {
                s.retry_count = attempt;
                s.status = StepStatus::Running;
            });

            LogPipeline::log_step(
                &app,
                LogLevel::Warn,
                &task_id,
                &step_id,
                LogSource::System,
                &format!("Retrying in {delay}s (attempt {attempt}/{max_attempts})…"),
            );

            tokio::time::sleep(Duration::from_secs(delay)).await;
            continue;
        }

        // Step failed — update status.
        update_step(&app, &task_arc, &step_id, |s| {
            s.status = StepStatus::Failed {
                error: err_msg.clone(),
            };
            s.finished_at = Some(now_ms());
            s.exit_code = exit_code;
            s.retry_count = attempt;
        });
        emit_progress(&app, &task_arc, None);

        if step.on_error == OnErrorStrategy::Skip {
            update_step(&app, &task_arc, &step_id, |s| {
                s.status = StepStatus::Skipped;
            });
            return StepOutcome::Skipped { step_id };
        }

        return StepOutcome::Failed {
            step_id,
            error: err_msg,
        };
    }
}

/// Main async executor.
///
/// Phase 3: builds a DAG from the recipe steps and executes layers in
/// parallel.  Within a layer, all independent steps run concurrently via
/// `tokio::task::JoinSet`.  Pause / cancel control messages are processed
/// between scheduling rounds.
pub async fn run_task_executor(
    app: AppHandle,
    task_arc: Arc<Mutex<Task>>,
    recipe: Recipe,
    vars: HashMap<String, String>,
    mut ctrl_rx: mpsc::Receiver<TaskControl>,
) {
    // Build DAG — abort immediately if the recipe has invalid dependencies.
    let graph = match TaskGraph::build(&recipe.steps) {
        Ok(g) => g,
        Err(e) => {
            set_task_status(&app, &task_arc, TaskStatus::Failed, Some(e.to_string()));
            return;
        }
    };

    set_task_status(&app, &task_arc, TaskStatus::Running, None);

    // Merge recipe vars with caller-supplied vars (caller wins).
    let mut merged_vars = recipe.vars.clone();
    merged_vars.extend(vars);

    let task_id: String = task_arc.lock().unwrap().id.clone();
    let total_steps = recipe.steps.len();

    // Track execution state.
    let mut completed: HashSet<String> = HashSet::new();
    let mut running: HashSet<String> = HashSet::new();
    let mut task_failed = false;
    let mut fail_error: Option<String> = None;
    let mut paused = false;

    // JoinSet holds all concurrently executing step futures.
    let mut join_set: JoinSet<StepOutcome> = JoinSet::new();

    loop {
        // ── Drain non-blocking control messages ─────────────────────────
        loop {
            match ctrl_rx.try_recv() {
                Ok(TaskControl::Cancel) => {
                    join_set.abort_all();
                    cancel_all_pending(&app, &task_arc, &completed, &running);
                    set_task_status(&app, &task_arc, TaskStatus::Cancelled, None);
                    return;
                }
                Ok(TaskControl::Pause) => paused = true,
                Ok(TaskControl::Resume) => {
                    if paused {
                        paused = false;
                        set_task_status(&app, &task_arc, TaskStatus::Running, None);
                    }
                }
                Err(_) => break,
            }
        }

        // ── If paused and nothing is running, wait for a control message ─
        if paused && join_set.is_empty() {
            set_task_status(&app, &task_arc, TaskStatus::Paused, None);
            loop {
                match ctrl_rx.recv().await {
                    Some(TaskControl::Resume) => {
                        paused = false;
                        set_task_status(&app, &task_arc, TaskStatus::Running, None);
                        break;
                    }
                    Some(TaskControl::Cancel) | None => {
                        cancel_all_pending(&app, &task_arc, &completed, &running);
                        set_task_status(&app, &task_arc, TaskStatus::Cancelled, None);
                        return;
                    }
                    Some(TaskControl::Pause) => { /* already paused */ }
                }
            }
        }

        // ── Schedule newly ready steps (if not failed / paused) ─────────
        if !task_failed && !paused {
            let ready = graph.get_ready_steps(&completed, &running);
            for step_id in ready {
                let recipe_step = match recipe.steps.iter().find(|s| s.id == step_id) {
                    Some(rs) => rs.clone(),
                    None => continue,
                };
                let step = substitute_step(&recipe_step, &merged_vars);
                running.insert(step_id.clone());

                let app_c = app.clone();
                let task_arc_c = task_arc.clone();
                let tid = task_id.clone();
                join_set.spawn(run_step(app_c, task_arc_c, tid, step));
            }
        }

        // ── Check termination ────────────────────────────────────────────
        if join_set.is_empty() {
            if task_failed {
                set_task_status(&app, &task_arc, TaskStatus::Failed, fail_error);
            } else if completed.len() == total_steps {
                set_task_status(&app, &task_arc, TaskStatus::Success, None);
            } else {
                // Some steps couldn't be reached (dependency on a failed step).
                set_task_status(
                    &app,
                    &task_arc,
                    TaskStatus::Failed,
                    Some("Some steps were unreachable due to upstream failures".into()),
                );
            }
            return;
        }

        // ── Wait for the next step to finish (or a control message) ──────
        tokio::select! {
            Some(result) = join_set.join_next() => {
                match result {
                    Ok(StepOutcome::Success { step_id }) |
                    Ok(StepOutcome::Skipped { step_id }) => {
                        running.remove(&step_id);
                        completed.insert(step_id);
                    }
                    Ok(StepOutcome::Failed { step_id, error }) => {
                        running.remove(&step_id);
                        task_failed = true;
                        fail_error = Some(error);
                        // Abort all other in-flight steps.
                        join_set.abort_all();
                        // Drain remaining join results so they don't leak.
                        while join_set.join_next().await.is_some() {}
                        cancel_all_pending(&app, &task_arc, &completed, &running);
                        let _ = step_id; // step status already set in run_step
                        // Will terminate on next loop iteration.
                    }
                    Err(_) => {
                        // Task was aborted (cancel path); outcome handled above.
                    }
                }
            }
            msg = ctrl_rx.recv() => {
                match msg {
                    Some(TaskControl::Cancel) => {
                        join_set.abort_all();
                        while join_set.join_next().await.is_some() {}
                        cancel_all_pending(&app, &task_arc, &completed, &running);
                        set_task_status(&app, &task_arc, TaskStatus::Cancelled, None);
                        return;
                    }
                    Some(TaskControl::Pause) => {
                        paused = true;
                        // Let running steps finish naturally before fully pausing.
                        set_task_status(&app, &task_arc, TaskStatus::Paused, None);
                    }
                    Some(TaskControl::Resume) => {
                        if paused {
                            paused = false;
                            set_task_status(&app, &task_arc, TaskStatus::Running, None);
                        }
                    }
                    None => { /* channel closed */ }
                }
            }
        }
    }
}
/// Mark all Pending/Waiting steps (not in `completed` or `running`) as
/// Cancelled.  Used when a task is aborted.
fn cancel_all_pending(
    app: &AppHandle,
    task_arc: &Arc<Mutex<Task>>,
    completed: &HashSet<String>,
    running: &HashSet<String>,
) {
    let ids: Vec<String> = {
        let task = task_arc.lock().unwrap();
        task.steps
            .iter()
            .filter(|s| {
                !completed.contains(&s.id)
                    && !running.contains(&s.id)
                    && matches!(s.status, StepStatus::Pending | StepStatus::Waiting)
            })
            .map(|s| s.id.clone())
            .collect()
    };
    for id in ids {
        update_step(app, task_arc, &id, |s| {
            s.status = StepStatus::Cancelled;
        });
    }
}
