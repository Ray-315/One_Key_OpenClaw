use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Plugin manifest (parsed from plugin.toml)
// ---------------------------------------------------------------------------

/// Metadata read from a plugin's `plugin.toml` manifest file.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginManifest {
    /// Unique plugin identifier.
    pub id: String,
    /// Human-readable name.
    pub name: String,
    /// Semantic version string.
    pub version: String,
    /// Author / maintainer.
    #[serde(default)]
    pub author: Option<String>,
    /// Plugin capabilities (one or more of the supported types).
    #[serde(default)]
    pub types: Vec<PluginType>,
    /// Entry file (reserved for future WASM support).
    #[serde(default)]
    pub entry: Option<String>,
    /// Permission declarations.
    #[serde(default)]
    pub permissions: PluginPermissions,
}

/// Supported plugin capability types.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PluginType {
    /// Provide recipe lists (e.g. community marketplace).
    RecipeProvider,
    /// Custom environment probe.
    EnvProbe,
    /// Custom step executor.
    StepExecutor,
    /// Extended error diagnosis rules.
    ErrorRule,
    /// Log output sink.
    LogSink,
}

/// Permission flags declared in a plugin manifest.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginPermissions {
    #[serde(default)]
    pub network: bool,
    #[serde(default)]
    pub filesystem: bool,
} // ---------------------------------------------------------------------------
  // Runtime plugin info (exposed to frontend)
  // ---------------------------------------------------------------------------

/// Lightweight view of a loaded plugin exposed to the frontend.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginInfo {
    pub id: String,
    pub name: String,
    pub version: String,
    pub author: Option<String>,
    pub types: Vec<PluginType>,
    pub enabled: bool,
}

impl From<&PluginManifest> for PluginInfo {
    fn from(m: &PluginManifest) -> Self {
        Self {
            id: m.id.clone(),
            name: m.name.clone(),
            version: m.version.clone(),
            author: m.author.clone(),
            types: m.types.clone(),
            enabled: true,
        }
    }
}
