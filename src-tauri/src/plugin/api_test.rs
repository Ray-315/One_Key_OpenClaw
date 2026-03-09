#[cfg(test)]
mod tests {
    use crate::plugin::api::{PluginInfo, PluginManifest, PluginPermissions, PluginType};

    fn sample_manifest() -> PluginManifest {
        PluginManifest {
            id: "test-plugin".into(),
            name: "Test Plugin".into(),
            version: "1.0.0".into(),
            author: Some("Author".into()),
            types: vec![PluginType::RecipeProvider, PluginType::EnvProbe],
            entry: None,
            permissions: PluginPermissions {
                network: true,
                filesystem: false,
            },
        }
    }

    #[test]
    fn manifest_to_plugin_info() {
        let manifest = sample_manifest();
        let info: PluginInfo = PluginInfo::from(&manifest);
        assert_eq!(info.id, "test-plugin");
        assert_eq!(info.name, "Test Plugin");
        assert_eq!(info.version, "1.0.0");
        assert!(info.enabled);
        assert_eq!(info.types.len(), 2);
    }

    #[test]
    fn default_permissions_are_restrictive() {
        let perms = PluginPermissions::default();
        assert!(!perms.network);
        assert!(!perms.filesystem);
    }

    #[test]
    fn plugin_type_equality() {
        assert_eq!(PluginType::RecipeProvider, PluginType::RecipeProvider);
        assert_ne!(PluginType::RecipeProvider, PluginType::EnvProbe);
    }

    #[test]
    fn manifest_serialization_roundtrip() {
        let manifest = sample_manifest();
        let json = serde_json::to_string(&manifest).unwrap();
        let parsed: PluginManifest = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.id, manifest.id);
        assert_eq!(parsed.name, manifest.name);
        assert_eq!(parsed.types.len(), manifest.types.len());
    }
}
