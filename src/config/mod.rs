use std::fs;
use std::path::Path;

use serde::Deserialize;

use crate::errors::AppError;

// TODO(port-config): schéma détaillé du config.toml global — proposition dédiée.
#[derive(Debug, Default, Deserialize)]
pub struct GlobalConfig {}

impl GlobalConfig {
    pub fn load_or_default(path: &Path) -> Result<Self, AppError> {
        match fs::read_to_string(path) {
            Ok(contents) => Ok(toml::from_str(&contents)?),
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(Self::default()),
            Err(err) => Err(AppError::Io(err)),
        }
    }
}
