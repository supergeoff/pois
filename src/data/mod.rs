use std::fs;
use std::path::Path;

use crate::errors::AppError;

const SUBDIRS: &[&str] = &["agents", "honcho", "logs"];

/// Ensure the `/data/` layout exists, creating missing subdirectories.
///
/// Does not touch `config.toml` — the global config file is created lazily
/// by whichever CLI subcommand first needs to write it.
pub fn ensure_layout(root: &Path) -> Result<(), AppError> {
    fs::create_dir_all(root)?;
    for sub in SUBDIRS {
        fs::create_dir_all(root.join(sub))?;
    }
    Ok(())
}
