#![warn(clippy::pedantic)]

use std::fs;
use std::path::PathBuf;
use std::process;
use std::sync::mpsc;

use daemon::Daemon;
use tracing::Subscriber;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use walrus_core::commands::Commands;
use walrus_core::config::Config;
use walrus_core::ipc;
use walrus_core::utils;
use walrus_core::utils::DirError;
use walrus_core::utils::Dirs;

mod daemon;
mod transition;

fn main() {
    // Start logging to file (and journald if it's available).
    let log_dir = match utils::get_app_dir_with(Dirs::State, "logs") {
        Ok(p) => p,
        Err(DirError::DoesNotExist(path)) => {
            fs::create_dir_all(&path).expect("Error creating directories for log file");
            path
        }
        Err(e @ (DirError::InvalidPath(_) | DirError::IoError(_) | DirError::MissingVar(_))) => {
            tracing::error!("Not logging to file: {e}");
            process::exit(1);
        }
    };
    daemon_logger(log_dir).init();

    let config = Config::new().unwrap_or_else(|e| {
        tracing::error!("Error in config: {e}");
        tracing::warn!("Falling back to default config...");
        Config::default()
    });

    let mut daemon = Daemon::new(config);
    if daemon.queue.is_empty() {
        tracing::info!("Queue is empty, exiting...");
        return;
    }

    let (tx, rx) = mpsc::channel();

    // Start watching the Config.toml for changes. This spawns a detached thread.
    // NOTE: In the future I might want to return a join handle here to clean up and retry on fail.
    match utils::get_config_file("config.toml") {
        Ok(path) => match Config::watch(path, tx.clone()) {
            Ok(()) => tracing::debug!("Starting inotify service"),
            Err(e) => {
                tracing::error!("Error starting inotify watcher: {e}");
                tracing::warn!(
                    "Unable to start inotify service: config hot reloading will not work"
                );
            }
        },
        Err(e) => {
            tracing::error!("Could not get config file: {e}");
            tracing::warn!("Unable to start inotify service: config hot reloading will not work");
        }
    }

    let ctrlc_tx = tx.clone();
    ctrlc::set_handler(move || {
        let _ = ctrlc_tx.send(Commands::Shutdown);
    })
    .expect("Error setting Ctrl-C handler");

    let _ipc = ipc::start_server(tx.clone());

    daemon.run(&rx);
}

#[must_use]
pub fn daemon_logger(log_dir: PathBuf) -> impl Subscriber {
    let file_appender = tracing_appender::rolling::daily(log_dir, "walrus");
    let file_layer = tracing_subscriber::fmt::layer()
        .with_writer(file_appender)
        .with_ansi(false);
    let subscriber = tracing_subscriber::registry().with(file_layer);

    // Some systems might not have journald or connecting to the socket could fail for whatever
    // reason. If it does fail this will return None and the layer will just do nothing.
    // https://docs.rs/tracing-subscriber/0.3.20/tracing_subscriber/layer/index.html#runtime-configuration-with-layers
    let journald_layer = tracing_journald::layer().ok();
    subscriber.with(journald_layer)
}
