use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::path::PathBuf;

use anyhow::Context;
use clap::Args;
use tracing_subscriber::EnvFilter;

use crate::data;
use crate::gateway::{self, auth::BasicAuth};

#[derive(Debug, Args)]
pub struct GatewayArgs {
    /// Port to listen on. Railway and similar PaaS set this via the `PORT` env var.
    #[arg(long, env = "PORT", default_value_t = 8080)]
    pub port: u16,

    /// Root directory for persistent state. Railway mounts a volume here.
    #[arg(long, env = "POIS_DATA_DIR", default_value = "/data")]
    pub data_dir: PathBuf,
}

pub async fn run(args: GatewayArgs) -> anyhow::Result<()> {
    init_tracing().context("failed to initialise tracing")?;

    let auth = BasicAuth::from_env().context("failed to load dashboard credentials")?;

    data::ensure_layout(&args.data_dir).with_context(|| {
        format!(
            "failed to initialise data layout at {}",
            args.data_dir.display()
        )
    })?;

    let bind_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), args.port);
    gateway::serve(bind_addr, auth, args.data_dir).await?;
    Ok(())
}

/// Output format of the tracing layer selected by `POIS_LOG_FORMAT`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LogFormat {
    /// Human-readable, multi-line layout (default when the env var is unset).
    Pretty,
    /// Structured single-line JSON objects (production default via Dockerfile).
    Json,
}

/// Returned when `POIS_LOG_FORMAT` holds a value that is neither
/// `"json"` nor `"pretty"` (case-insensitive).
#[derive(Debug)]
struct InvalidLogFormat(String);

impl std::fmt::Display for InvalidLogFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "invalid POIS_LOG_FORMAT value {:?}: expected \"json\" or \"pretty\"",
            self.0
        )
    }
}

impl std::error::Error for InvalidLogFormat {}

impl LogFormat {
    /// Parse the value of `POIS_LOG_FORMAT`.
    ///
    /// * `None` or an empty / whitespace-only string → [`LogFormat::Pretty`] (default).
    /// * `"json"` (case-insensitive) → [`LogFormat::Json`].
    /// * `"pretty"` (case-insensitive) → [`LogFormat::Pretty`].
    /// * any other value → [`InvalidLogFormat`] containing the offending string.
    fn from_env_value(raw: Option<&str>) -> Result<Self, InvalidLogFormat> {
        let trimmed = raw.map(str::trim).filter(|s| !s.is_empty());
        match trimmed {
            None => Ok(Self::Pretty),
            Some(v) if v.eq_ignore_ascii_case("json") => Ok(Self::Json),
            Some(v) if v.eq_ignore_ascii_case("pretty") => Ok(Self::Pretty),
            Some(other) => Err(InvalidLogFormat(other.to_string())),
        }
    }
}

fn init_tracing() -> anyhow::Result<()> {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    let raw = std::env::var("POIS_LOG_FORMAT").ok();
    let format = LogFormat::from_env_value(raw.as_deref()).map_err(|e| anyhow::anyhow!("{e}"))?;

    let builder = tracing_subscriber::fmt().with_env_filter(filter);
    match format {
        LogFormat::Json => builder.json().init(),
        LogFormat::Pretty => builder.pretty().init(),
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{InvalidLogFormat, LogFormat};

    #[test]
    fn from_env_value_none_defaults_to_pretty() {
        assert_eq!(LogFormat::from_env_value(None).unwrap(), LogFormat::Pretty);
    }

    #[test]
    fn from_env_value_empty_string_defaults_to_pretty() {
        assert_eq!(
            LogFormat::from_env_value(Some("")).unwrap(),
            LogFormat::Pretty
        );
    }

    #[test]
    fn from_env_value_whitespace_only_defaults_to_pretty() {
        assert_eq!(
            LogFormat::from_env_value(Some("   ")).unwrap(),
            LogFormat::Pretty
        );
    }

    #[test]
    fn from_env_value_json_lowercase_returns_json() {
        assert_eq!(
            LogFormat::from_env_value(Some("json")).unwrap(),
            LogFormat::Json
        );
    }

    #[test]
    fn from_env_value_json_is_case_insensitive() {
        assert_eq!(
            LogFormat::from_env_value(Some("JSON")).unwrap(),
            LogFormat::Json
        );
        assert_eq!(
            LogFormat::from_env_value(Some("Json")).unwrap(),
            LogFormat::Json
        );
    }

    #[test]
    fn from_env_value_pretty_is_case_insensitive() {
        assert_eq!(
            LogFormat::from_env_value(Some("pretty")).unwrap(),
            LogFormat::Pretty
        );
        assert_eq!(
            LogFormat::from_env_value(Some("PRETTY")).unwrap(),
            LogFormat::Pretty
        );
    }

    #[test]
    fn from_env_value_unknown_is_rejected_with_value_in_error() {
        let err = LogFormat::from_env_value(Some("xml")).unwrap_err();
        let InvalidLogFormat(value) = &err;
        assert_eq!(value, "xml");
        let rendered = err.to_string();
        assert!(
            rendered.contains("xml"),
            "error should name the value: {rendered}"
        );
        assert!(
            rendered.contains("POIS_LOG_FORMAT") || rendered.contains("json"),
            "error should help the operator: {rendered}"
        );
    }
}
