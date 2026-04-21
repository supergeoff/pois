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
    init_tracing();

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

fn init_tracing() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    let json = std::env::var("POIS_LOG_FORMAT").is_ok_and(|v| v.eq_ignore_ascii_case("json"));

    let builder = tracing_subscriber::fmt().with_env_filter(filter);
    if json {
        builder.json().init();
    } else {
        builder.compact().init();
    }
}
