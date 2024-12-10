use clap::Parser;
use log::{debug, error, LevelFilter};
use std::sync::mpsc;
use walrus::{
    commands::{Cli, Commands},
    config::Config,
    daemon::Daemon,
    ipc::{self, send_ipc_command},
    utils::init_logger,
};

fn main() {
    init_logger(LevelFilter::Trace).unwrap_or_else(|e| {
        error!("Failed to initialize logger: {e}\nContinuing without loggging...");
    });

    let config = Config::from("config.toml").unwrap_or_default();

    log::set_max_level(config.get_debug_level());
    debug!("Logging with log level: {}", config.get_debug_level());

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

    let mut walrus = Daemon::new(config).expect("Fatal: failed to initialize Walrus Daemon");
    if walrus.queue.is_empty() {
        error!("Queue is empty, exiting...");
        return;
    }

    let (tx, rx) = mpsc::channel();

    let ctrlc_tx = tx.clone();
    ctrlc::set_handler(move || {
        let _ = ctrlc_tx.send(Commands::Shutdown);
    })
    .expect("Error setting Ctrl-C handler");

    let ipc_tx = tx.clone();
    if let Err(e) = ipc::setup_ipc(ipc_tx) {
        error!("Failed to start IPC server: {e:#?}");
        return;
    }
    walrus.run(rx);
}
