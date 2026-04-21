use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("missing required environment variable: {0}")]
    MissingEnv(&'static str),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("failed to parse TOML config: {0}")]
    Config(#[from] toml::de::Error),

    #[error("failed to bind listener: {0}")]
    Bind(std::io::Error),

    #[error("template rendering error: {0}")]
    Template(#[from] askama::Error),
}
