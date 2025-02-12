#![warn(clippy::pedantic)]

use std::sync::mpsc;

use clap::Parser;
use log::{debug, error, warn, LevelFilter};
use walrus::{
    commands::{Cli, Commands},
    config::Config,
    daemon::Daemon,
    ipc, utils,
};

fn main() {
    utils::init_logger(LevelFilter::Trace).unwrap_or_else(|e| eprintln!("{e}"));
    log::set_max_level(LevelFilter::Info);

    let config = Config::new().unwrap_or_else(|e| {
        error!("Error in config: {e}");
        warn!("Falling back to default config...");
        Config::default()
    });

    let cli = Cli::parse();
    if let Some(cmd) = &cli.command {
        match cmd {
            Commands::Config => {
                debug!("Printing config to stdout...");
                return println!("{config}");
            }
            ipc_cmd => {
                debug!("Attempting to send {:?} command via IPC...", ipc_cmd);
                return ipc::send_ipc_command(*ipc_cmd)
                    .unwrap_or_else(|e| error!("Error sending command to running instance: {e}"));
            }
        }
    }

    log::set_max_level(config.debug());
    debug!("Logging with log level: {}", config.debug());

    let mut daemon = Daemon::new(config).expect("Fatal: failed to initialise Walrus Daemon");
    if daemon.queue.is_empty() {
        error!("Queue is empty, exiting...");
        return;
    }

    let (tx, rx) = mpsc::channel();

    match utils::get_config_file("config.toml") {
        Ok(path) => match Config::watch(path, tx.clone()) {
            Ok(()) => debug!("Starting inotify service"),
            Err(e) => {
                error!("{e}");
                warn!("Unable to start inotify service: config hot reloading will not work");
            }
        },
        Err(e) => {
            error!("{e}");
            warn!("Unable to start inotify service: config hot reloading will not work");
        }
    }

    let ctrlc_tx = tx.clone();
    ctrlc::set_handler(move || {
        let _ = ctrlc_tx.send(Commands::Shutdown);
    })
    .expect("Error setting Ctrl-C handler");

    if let Err(e) = ipc::setup_ipc(tx.clone()) {
        error!("Failed to start IPC server: {e:#?}");
        return;
    }
    daemon.run(rx);
}
