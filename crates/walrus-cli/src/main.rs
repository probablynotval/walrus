#![warn(clippy::pedantic)]

use clap::Parser;
use tracing_subscriber::EnvFilter;
use walrus_core::commands::Cli;
use walrus_core::commands::Commands;
use walrus_core::config::Config;
use walrus_core::ipc;

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .pretty()
        .init();

    let config = Config::new().unwrap_or_else(|e| {
        tracing::error!("Error in config: {e}");
        tracing::warn!("Falling back to default config...");
        Config::default()
    });

    let cli = Cli::parse();
    if let Some(cmd) = &cli.command {
        match cmd {
            Commands::Config => {
                tracing::debug!("Printing config to stdout...");
                tracing::debug!("{config}");
                println!("{config}");
            }
            ipc_cmd => {
                tracing::debug!("Attempting to send {ipc_cmd:?} command via IPC...");
                ipc::send_command(ipc_cmd.clone()).unwrap_or_else(|e| {
                    tracing::error!("Error sending command to walrus-daemon instance: {e}");
                    tracing::error!("Is walrus-daemon running?");
                });
            }
        }
    }
}
