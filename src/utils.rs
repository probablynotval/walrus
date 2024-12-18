use chrono::{DurationRound, Local, TimeDelta};
use directories::BaseDirs;
use log::{debug, error, trace, warn, LevelFilter};
use notify::{RecommendedWatcher, Watcher};
use simplelog::{ColorChoice, CombinedLogger, Config, TermLogger, TerminalMode, WriteLogger};
use std::{
    error::Error,
    fs::{self, File},
    path::{Path, PathBuf},
    sync::mpsc::{self, Sender},
    thread,
    time::Duration,
};

use crate::commands::Commands;

pub const APPNAME: &str = "walrus";
pub const SOCKET_PATH: &str = "/tmp/walrus.sock";

fn get_appdata_dir<P: AsRef<Path>>(path: P) -> PathBuf {
    let base_dir =
        BaseDirs::new().expect("Failed to get base directory for appdata directory: {dir}");
    let dir = base_dir.data_dir().join(APPNAME).join(path);
    if !dir.exists() {
        fs::create_dir_all(&dir).expect("Failed to create appdata directory: {dir}");
    }
    dir
}

pub fn get_config_file<P: AsRef<Path>>(filename: P) -> PathBuf {
    let base_dir = BaseDirs::new().expect("Failed to get base directory for config directory");
    let config_dir = base_dir.config_dir().join(APPNAME);
    if !config_dir.exists() {
        fs::create_dir_all(&config_dir).expect("Failed to create config directory");
    }
    let config_file = config_dir.join(filename);
    if !config_file.exists() {
        File::create(&config_file).expect("Failed to create config file");
    }
    config_file
}

pub fn init_logger(log_level: LevelFilter) -> Result<(), Box<dyn Error>> {
    let log_dir = get_appdata_dir("logs");
    let log_name = Local::now()
        .duration_trunc(TimeDelta::try_seconds(1).unwrap())
        .unwrap()
        .to_rfc3339()
        + ".log";
    let log_path = log_dir.join(log_name);

    let tlogger = TermLogger::new(
        log_level,
        Config::default(),
        TerminalMode::Mixed,
        ColorChoice::Auto,
    );
    let wlogger = WriteLogger::new(log_level, Config::default(), File::create(&log_path)?);
    CombinedLogger::init(vec![tlogger, wlogger])?;
    Ok(())
}

pub fn normalize_duration(base_duration: f64, width: f64, height: f64, angle_degrees: f64) -> f64 {
    let theta = angle_degrees.to_radians();
    let distance_at_angle = (width * theta.cos().abs()) + (height * theta.sin().abs());
    trace!("DistAtAngle: {distance_at_angle}");
    let diagonal_distance = (width.powi(2) + height.powi(2)).sqrt();
    trace!("DiagDist: {diagonal_distance}");
    let ratio = diagonal_distance / distance_at_angle;
    base_duration * ratio
}

pub fn watch<P: AsRef<Path>>(path: P, tx: Sender<Commands>) -> notify::Result<()> {
    debug!("Starting watcher...");
    let (internal_tx, rx) = mpsc::channel();
    let mut watcher = RecommendedWatcher::new(
        internal_tx,
        notify::Config::default()
            .with_compare_contents(true)
            .with_poll_interval(Duration::from_secs(1)),
    )?;

    let abs_path = fs::canonicalize(path.as_ref())?;

    watcher.watch(abs_path.as_ref(), notify::RecursiveMode::NonRecursive)?;

    thread::spawn(move || -> notify::Result<()> {
        let mut watcher = watcher;

        while let Ok(event_res) = rx.recv() {
            match event_res {
                Ok(event) => {
                    debug!("File event: {event:?}");
                    if event.kind.is_modify() || event.kind.is_remove() {
                        tx.send(Commands::Reload).unwrap();
                        debug!("After Reload commmand");

                        if event.kind.is_remove() {
                            debug!("File removed, trying to re-establish watch");
                            thread::sleep(Duration::from_millis(200));
                            watcher
                                .watch(abs_path.as_ref(), notify::RecursiveMode::NonRecursive)?;
                        }
                    }
                }
                Err(e) => return Err(e),
            }
        }
        warn!("Watcher thread stopping");
        Ok(())
    });

    Ok(())
}
