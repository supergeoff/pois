pub mod gateway;

use clap::{Parser, Subcommand};

use crate::VERSION;

#[derive(Debug, Parser)]
#[command(name = "pois", version = VERSION, about = "Personal AI companion — CLI and web gateway")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Run the HTTP gateway (dashboard + health endpoint).
    Gateway(gateway::GatewayArgs),
}

pub async fn run(cli: Cli) -> anyhow::Result<()> {
    match cli.command {
        Command::Gateway(args) => gateway::run(args).await,
    }
}
