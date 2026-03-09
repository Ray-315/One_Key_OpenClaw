#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, Ordering};
use tauri::{AppHandle, Emitter};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LogEntry {
    pub id: u64,
    pub task_id: String,
    pub step_id: Option<String>,
    pub level: LogLevel,
    pub message: String,
    pub timestamp: u64,
    pub source: LogSource,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum LogSource {
    Stdout,
    Stderr,
    System,
    Plugin { plugin_id: String },
}

static LOG_ID_COUNTER: AtomicU64 = AtomicU64::new(1);

fn next_log_id() -> u64 {
    LOG_ID_COUNTER.fetch_add(1, Ordering::Relaxed)
}

fn now_millis() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

pub struct LogPipeline;

impl LogPipeline {
    /// Create a system-level log entry and emit it via Tauri events.
    pub fn log_system(app_handle: &AppHandle, level: LogLevel, message: impl Into<String>) {
        let entry = LogEntry {
            id: next_log_id(),
            task_id: "system".to_string(),
            step_id: None,
            level,
            message: message.into(),
            timestamp: now_millis(),
            source: LogSource::System,
        };

        // Best-effort emit; ignore errors (e.g. no active window).
        let _ = app_handle.emit("log://entry", &entry);
    }

    /// Create a step-scoped log entry and emit it via Tauri events.
    pub fn log_step(
        app_handle: &AppHandle,
        level: LogLevel,
        task_id: impl Into<String>,
        step_id: impl Into<String>,
        source: LogSource,
        message: impl Into<String>,
    ) {
        let entry = LogEntry {
            id: next_log_id(),
            task_id: task_id.into(),
            step_id: Some(step_id.into()),
            level,
            message: message.into(),
            timestamp: now_millis(),
            source,
        };
        let _ = app_handle.emit("log://entry", &entry);
    }
}
