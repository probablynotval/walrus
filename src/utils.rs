use chrono::{DurationRound, Local, TimeDelta};
use directories::BaseDirs;
use log::LevelFilter;
use simplelog::{ColorChoice, CombinedLogger, Config, TermLogger, TerminalMode, WriteLogger};
use std::{
    error::Error,
    fs::{self, File},
    path::PathBuf,
};

const APPNAME: &str = "walrus";

fn get_appdata_dir(dir: &str) -> PathBuf {
    let base_dir =
        BaseDirs::new().expect("Failed to get base directory for appdata directory: {dir}");
    let xdir = base_dir.data_dir().join(format!("{APPNAME}/{dir}"));
    if !xdir.exists() {
        fs::create_dir_all(&xdir).expect("Failed to create appdata directory: {dir}");
    }
    xdir
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
    let diagonal_distance = (width.powi(2) + height.powi(2)).sqrt();
    let ratio = diagonal_distance / distance_at_angle;
    base_duration * ratio
}
