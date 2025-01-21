#![warn(clippy::pedantic)]

use clap::Parser;
use log::{debug, error, LevelFilter};
use std::sync::mpsc;
use walrus::{
    commands::{Cli, Commands},
    config::Config,
    daemon::Daemon,
    ipc::{self, send_ipc_command},
    utils::{get_config_file, init_logger},
};

fn main() {
    init_logger(LevelFilter::Trace).unwrap_or_else(|e| {
        eprintln!("Failed to initialize logger: {e}\nContinuing without loggging...");
    });
    log::set_max_level(LevelFilter::Info);

    let config = Config::new(None).unwrap_or_default();

    let cli = Cli::parse();
    if let Some(cmd) = &cli.command {
        match cmd {
            Commands::Config => {
                debug!("Printing config to stdout...");
                return println!("{config}");
            }
            Commands::Next => {
                debug!("Attempting to send Next command via IPC...");
                return send_ipc_command(Commands::Next)
                    .unwrap_or_else(|e| error!("Error sending command to running instance: {e}"));
            }
            Commands::Pause => {
                debug!("Attempting to send Pause command via IPC...");
                return send_ipc_command(Commands::Pause)
                    .unwrap_or_else(|e| error!("Error sending command to running instance: {e}"));
            }
            Commands::Previous => {
                debug!("Attempting to send Previous command via IPC...");
                return send_ipc_command(Commands::Previous)
                    .unwrap_or_else(|e| error!("Error sending command to running instance: {e}"));
            }
            Commands::Reload => {
                debug!("Attempting to send Reload command via IPC...");
                return send_ipc_command(Commands::Reload)
                    .unwrap_or_else(|e| error!("Error sending command to running instance: {e}"));
            }
            Commands::Resume => {
                debug!("Attempting to send Resume command via IPC...");
                return send_ipc_command(Commands::Resume)
                    .unwrap_or_else(|e| error!("Error sending command to running instance: {e}"));
            }
            Commands::Shutdown => {
                debug!("Attempting to send Shutdown command via IPC...");
                return send_ipc_command(Commands::Shutdown)
                    .unwrap_or_else(|e| error!("Error sending command to running instance: {e}"));
            }
        }
    }

    log::set_max_level(config.debug());
    debug!("Logging with log level: {}", config.debug());

    let mut walrus = Daemon::new(config).expect("Fatal: failed to initialize Walrus Daemon");
    if walrus.queue.is_empty() {
        error!("Queue is empty, exiting...");
        return;
    }

    let (tx, rx) = mpsc::channel();

    Config::watch(get_config_file("config.toml"), tx.clone()).expect("Error watching config.toml");

    let ctrlc_tx = tx.clone();
    ctrlc::set_handler(move || {
        let _ = ctrlc_tx.send(Commands::Shutdown);
    })
    .expect("Error setting Ctrl-C handler");

    if let Err(e) = ipc::setup_ipc(tx.clone()) {
        error!("Failed to start IPC server: {e:#?}");
        return;
    }
    walrus.run(rx);
}
