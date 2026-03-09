#[cfg(test)]
mod tests {
    use crate::env::platform::{display_name_for, install_hint_for};

    #[test]
    fn display_names_for_known_tools() {
        assert_eq!(display_name_for("node"), "Node.js");
        assert_eq!(display_name_for("git"), "Git");
        assert_eq!(display_name_for("python"), "Python");
        assert_eq!(display_name_for("rustc"), "Rust Compiler");
        assert_eq!(display_name_for("docker"), "Docker");
    }

    #[test]
    fn display_name_unknown_returns_id() {
        assert_eq!(display_name_for("unknown_tool"), "unknown_tool");
    }

    #[test]
    fn install_hint_node_has_all_platforms() {
        let hint = install_hint_for("node");
        assert!(hint.macos.is_some());
        assert!(hint.linux.is_some());
        assert!(hint.windows.is_some());
        assert!(hint.macos.unwrap().contains("brew"));
        assert!(hint.linux.unwrap().contains("apt"));
        assert!(hint.windows.unwrap().contains("winget"));
    }

    #[test]
    fn install_hint_unknown_returns_empty() {
        let hint = install_hint_for("unknown");
        assert!(hint.macos.is_none());
        assert!(hint.linux.is_none());
        assert!(hint.windows.is_none());
    }

    #[test]
    fn install_hint_all_tools_have_hints() {
        for tool in &["node", "git", "python", "rustc", "docker"] {
            let hint = install_hint_for(tool);
            assert!(hint.macos.is_some(), "missing macos hint for {tool}");
            assert!(hint.linux.is_some(), "missing linux hint for {tool}");
            assert!(hint.windows.is_some(), "missing windows hint for {tool}");
        }
    }
}
