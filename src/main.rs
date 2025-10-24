#![warn(clippy::pedantic)]

use std::sync::mpsc;

use clap::Parser;
use log::{LevelFilter, debug, error, warn};
use walrus::{
    commands::{Cli, Commands},
    config::Config,
    daemon::Daemon,
    ipc, utils,
};

fn main() {
    let config = Config::new().unwrap_or_else(|e| {
        error!("Error in config: {e}");
        warn!("Falling back to default config...");
        Config::default()
    });

    log::set_max_level(config.debug());
    debug!("Logging with log level: {}", config.debug());

    let cli = Cli::parse();
    if let Some(cmd) = &cli.command {
        utils::init_term_logger(LevelFilter::Trace)
            .unwrap_or_else(|e| eprintln!("Error initialising terminal logger: {e}"));

        match cmd {
            Commands::Config => {
                debug!("Printing config to stdout...");
                return println!("{config}");
            }
            ipc_cmd => {
                debug!("Attempting to send {ipc_cmd:?} command via IPC...");
                return ipc::send_ipc_command(*ipc_cmd)
                    .unwrap_or_else(|e| error!("Error sending command to running instance: {e}"));
            }
        }
    }

    utils::init_logger(LevelFilter::Trace).expect("Error initialising logger");
    // let file =
    //     utils::init_write_logger(LevelFilter::Trace).expect("Error initialising file logger");
    // debug!("Logging to file: {}", file.display());

    let mut daemon = Daemon::new(config);
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

    let _ipc = ipc::setup_ipc(tx.clone());

    daemon.run(rx);
}
