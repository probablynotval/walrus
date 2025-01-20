use chrono::{DurationRound, Local, TimeDelta};
use directories::BaseDirs;
use log::{debug, LevelFilter};
use simplelog::{ColorChoice, CombinedLogger, Config, TermLogger, TerminalMode, WriteLogger};
use std::{
    error::Error,
    fs::{self, File},
    path::{Path, PathBuf},
};

pub const APPNAME: &str = "walrus";
pub const SOCKET_PATH: &str = "/tmp/walrus.sock";
pub const SCREENWH: (f64, f64) = (2560.0, 1440.0);

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
    debug!("DistAtAngle: {distance_at_angle}");
    let diagonal_distance = (width.powi(2) + height.powi(2)).sqrt();
    let ratio = diagonal_distance / distance_at_angle;
    base_duration * ratio
}

pub fn decrement_index(index: usize, qlen: usize) -> usize {
    (index + qlen - 1) % qlen
}

pub fn increment_index(index: usize, qlen: usize) -> usize {
    (index + 1) % qlen
}
