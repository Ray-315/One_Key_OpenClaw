use serde::Serialize;

#[derive(Debug, thiserror::Error)]
#[allow(dead_code)]
pub enum AppError {
    #[error("Environment probe failed: {env_id} - {message}")]
    EnvProbeError { env_id: String, message: String },

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("System error: {0}")]
    Anyhow(#[from] anyhow::Error),
}

impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}
