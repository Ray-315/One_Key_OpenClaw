use crate::env::prober::InstallHint;

/// Returns platform-specific install hints for a given tool.
pub fn install_hint_for(id: &str) -> InstallHint {
    match id {
        "node" => InstallHint {
            macos: Some("brew install node".into()),
            linux: Some("sudo apt install nodejs npm".into()),
            windows: Some("winget install OpenJS.NodeJS.LTS".into()),
        },
        "git" => InstallHint {
            macos: Some("brew install git".into()),
            linux: Some("sudo apt install git".into()),
            windows: Some("winget install Git.Git".into()),
        },
        "python" => InstallHint {
            macos: Some("brew install python".into()),
            linux: Some("sudo apt install python3".into()),
            windows: Some("winget install Python.Python.3.12".into()),
        },
        "rustc" => InstallHint {
            macos: Some("curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh".into()),
            linux: Some("curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh".into()),
            windows: Some("winget install Rustlang.Rustup".into()),
        },
        "docker" => InstallHint {
            macos: Some("brew install --cask docker".into()),
            linux: Some("sudo apt install docker.io".into()),
            windows: Some("winget install Docker.DockerDesktop".into()),
        },
        _ => InstallHint {
            macos: None,
            linux: None,
            windows: None,
        },
    }
}

/// Returns the display name for a tool id.
pub fn display_name_for(id: &str) -> &str {
    match id {
        "node" => "Node.js",
        "git" => "Git",
        "python" => "Python",
        "rustc" => "Rust Compiler",
        "docker" => "Docker",
        _ => id,
    }
}
