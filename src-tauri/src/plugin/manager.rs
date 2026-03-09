use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::error::AppError;
use crate::plugin::api::{PluginInfo, PluginManifest};

/// Manages locally-installed plugins.
///
/// Plugins live under a `plugins/` directory.  Each sub-directory is expected
/// to contain a `plugin.toml` manifest that describes the plugin's
/// capabilities and metadata.
pub struct PluginManager {
    /// Loaded plugin manifests keyed by plugin id.
    plugins: HashMap<String, PluginManifest>,
    /// Root directory where plugin sub-directories are stored.
    plugin_dir: PathBuf,
}

impl PluginManager {
    /// Create a new PluginManager that scans `plugin_dir` for plugins.
    pub fn new(plugin_dir: PathBuf) -> Self {
        Self {
            plugins: HashMap::new(),
            plugin_dir,
        }
    }

    /// Scan `plugin_dir` for sub-directories that contain a `plugin.toml` and
    /// load their manifests.
    pub fn scan_plugins(&mut self) -> Result<usize, AppError> {
        if !self.plugin_dir.exists() {
            // No plugin directory yet – nothing to scan.
            return Ok(0);
        }

        let entries = std::fs::read_dir(&self.plugin_dir).map_err(AppError::IoError)?;

        let mut count = 0usize;
        for entry in entries {
            let entry = entry.map_err(AppError::IoError)?;
            let path = entry.path();
            if path.is_dir() {
                let manifest_path = path.join("plugin.toml");
                if manifest_path.exists() {
                    match self.load_manifest(&manifest_path) {
                        Ok(_) => count += 1,
                        Err(e) => {
                            eprintln!(
                                "[PluginManager] Failed to load plugin at {}: {e}",
                                manifest_path.display()
                            );
                        }
                    }
                }
            }
        }

        Ok(count)
    }

    /// Load a single plugin from the given `plugin.toml` path.
    pub fn load_plugin(&mut self, path: &Path) -> Result<PluginInfo, AppError> {
        let manifest = self.load_manifest(path)?;
        let info = PluginInfo::from(&manifest);
        self.plugins.insert(manifest.id.clone(), manifest);
        Ok(info)
    }

    /// Unload (remove) a plugin by its id.
    pub fn unload_plugin(&mut self, id: &str) -> Result<(), AppError> {
        if self.plugins.remove(id).is_some() {
            Ok(())
        } else {
            Err(AppError::PluginError {
                plugin_id: id.into(),
                message: "plugin not found".into(),
            })
        }
    }

    /// Return information about all loaded plugins.
    pub fn list_plugins(&self) -> Vec<PluginInfo> {
        self.plugins.values().map(PluginInfo::from).collect()
    }

    /// Look up a loaded plugin by id.
    pub fn get_plugin(&self, id: &str) -> Option<PluginInfo> {
        self.plugins.get(id).map(PluginInfo::from)
    }

    // -----------------------------------------------------------------------
    // Internal helpers
    // -----------------------------------------------------------------------

    fn load_manifest(&mut self, path: &Path) -> Result<PluginManifest, AppError> {
        let content = std::fs::read_to_string(path).map_err(AppError::IoError)?;
        let manifest: PluginManifest = toml::from_str(&content).map_err(|e| {
            AppError::PluginError {
                plugin_id: path.display().to_string(),
                message: format!("invalid plugin.toml: {e}"),
            }
        })?;

        if manifest.id.is_empty() {
            return Err(AppError::PluginError {
                plugin_id: path.display().to_string(),
                message: "plugin id must not be empty".into(),
            });
        }

        Ok(manifest)
    }
}
