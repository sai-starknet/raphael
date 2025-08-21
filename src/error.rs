use thiserror::Error;

#[derive(Error, Debug)]
pub enum SaiError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("TOML parsing error: {0}")]
    Toml(#[from] toml::de::Error),

    #[error("StarkNet error: {0}")]
    StarkNet(#[from] starknet::providers::ProviderError),

    #[error("Field element error: {0}")]
    FieldElement(String),

    #[error("Contract declaration failed: {0}")]
    DeclarationFailed(String),

    #[error("Contract deployment failed: {0}")]
    DeploymentFailed(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Account error: {0}")]
    Account(String),

    #[error("Missing required field: {0}")]
    MissingField(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),
}

pub type Result<T> = std::result::Result<T, SaiError>;
