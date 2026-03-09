#[cfg(test)]
mod tests {
    use crate::env::prober::EnvProber;

    #[test]
    fn parse_version_extracts_semver() {
        let v = EnvProber::parse_version("node", "v20.11.0");
        assert_eq!(v, Some("20.11.0".to_string()));
    }

    #[test]
    fn parse_version_extracts_from_full_output() {
        let v = EnvProber::parse_version("git", "git version 2.43.0");
        assert_eq!(v, Some("2.43.0".to_string()));
    }

    #[test]
    fn parse_version_handles_python_output() {
        let v = EnvProber::parse_version("python", "Python 3.12.1");
        assert_eq!(v, Some("3.12.1".to_string()));
    }

    #[test]
    fn parse_version_handles_rustc_output() {
        let v = EnvProber::parse_version("rustc", "rustc 1.77.2 (25ef9e3d8 2024-04-09)");
        assert_eq!(v, Some("1.77.2".to_string()));
    }

    #[test]
    fn parse_version_returns_none_for_no_version() {
        let v = EnvProber::parse_version("node", "no version info");
        assert!(v.is_none());
    }

    #[test]
    fn parse_version_strips_v_prefix() {
        let v = EnvProber::parse_version("node", "v18.0.0");
        assert_eq!(v, Some("18.0.0".to_string()));
    }

    #[test]
    fn probe_returns_missing_for_nonexistent_tool() {
        let item = EnvProber::probe("nonexistent_tool_xyz_12345");
        assert!(
            matches!(item.status, crate::env::prober::EnvStatus::Missing),
            "Expected Missing status for nonexistent tool"
        );
        assert!(item.version.is_none());
    }
}
