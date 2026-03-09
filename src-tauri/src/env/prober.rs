use std::path::PathBuf;
use std::process::Command;

use serde::{Deserialize, Serialize};

use crate::env::platform;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EnvItem {
    pub id: String,
    pub name: String,
    pub status: EnvStatus,
    pub version: Option<String>,
    pub required_version: Option<String>,
    pub path: Option<PathBuf>,
    pub install_hint: Option<InstallHint>,
    pub checked_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum EnvStatus {
    Ok,
    Missing,
    VersionMismatch { found: String, required: String },
    Error { message: String },
    Checking,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallHint {
    pub macos: Option<String>,
    pub windows: Option<String>,
    pub linux: Option<String>,
}

/// All tool IDs that the prober supports.
const TOOL_IDS: &[&str] = &["node", "git", "python", "rustc", "docker"];

fn now_millis() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

pub struct EnvProber;

impl EnvProber {
    /// Probe all supported environments and return an `EnvItem` for each.
    pub fn probe_all() -> Vec<EnvItem> {
        TOOL_IDS.iter().map(|id| Self::probe(id)).collect()
    }

    /// Probe a single environment by tool id.
    pub fn probe(id: &str) -> EnvItem {
        // For python we try python3 first, then python.
        let commands: Vec<&str> = match id {
            "python" => vec!["python3", "python"],
            _ => vec![id],
        };

        for cmd in &commands {
            if let Some(item) = Self::try_probe(id, cmd) {
                return item;
            }
        }

        // Not found at all
        EnvItem {
            id: id.to_string(),
            name: platform::display_name_for(id).to_string(),
            status: EnvStatus::Missing,
            version: None,
            required_version: None,
            path: None,
            install_hint: Some(platform::install_hint_for(id)),
            checked_at: now_millis(),
        }
    }

    /// Attempt to probe a single command. Returns `Some(EnvItem)` if the binary
    /// is found on PATH (regardless of whether version parsing succeeds).
    fn try_probe(id: &str, cmd: &str) -> Option<EnvItem> {
        let path = which::which(cmd).ok()?;
        let (status, version) = Self::detect_version(id, cmd, &path);

        Some(EnvItem {
            id: id.to_string(),
            name: platform::display_name_for(id).to_string(),
            status,
            version,
            required_version: None,
            path: Some(path),
            install_hint: Some(platform::install_hint_for(id)),
            checked_at: now_millis(),
        })
    }

    /// Run `<cmd> --version` and attempt to extract a semver-like version string.
    fn detect_version(id: &str, cmd: &str, path: &PathBuf) -> (EnvStatus, Option<String>) {
        let output = Command::new(path).arg("--version").output();

        match output {
            Ok(out) => {
                let text = String::from_utf8_lossy(&out.stdout).to_string()
                    + &String::from_utf8_lossy(&out.stderr);
                match Self::parse_version(id, &text) {
                    Some(ver) => (EnvStatus::Ok, Some(ver)),
                    None => (EnvStatus::Ok, None),
                }
            }
            Err(e) => (
                EnvStatus::Error {
                    message: format!("Failed to run {cmd} --version: {e}"),
                },
                None,
            ),
        }
    }

    /// Parse a version string from the output of `<tool> --version`.
    /// The `id` parameter is reserved for future tool-specific parsing.
    pub(crate) fn parse_version(_id: &str, output: &str) -> Option<String> {
        // Look for the first semver-like pattern (e.g. 1.2.3, 20.11.0)
        let re_like = output
            .split_whitespace()
            .find(|word| {
                let trimmed = word.trim_start_matches('v');
                trimmed.contains('.')
                    && trimmed
                        .chars()
                        .next()
                        .map(|c| c.is_ascii_digit())
                        .unwrap_or(false)
            })?
            .trim_start_matches('v')
            .trim_end_matches(',');

        Some(re_like.to_string())
    }
}
