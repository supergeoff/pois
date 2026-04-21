pub mod cli;
pub mod config;
pub mod data;
pub mod errors;
pub mod gateway;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
