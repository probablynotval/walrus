use std::{
    env,
    error::Error,
    fmt::{self, Display},
    fs::{self, File},
    io,
    path::{Path, PathBuf},
    str::FromStr,
};

use log::{LevelFilter, debug, error, info, warn};
use serde::{Deserialize, Deserializer};
use simplelog::{ColorChoice, CombinedLogger, Config, TermLogger, TerminalMode, WriteLogger};
use time::{OffsetDateTime, format_description::well_known};

use crate::config::{Resolution, TransitionFlavour};

pub const APPNAME: &str = "walrus";

pub enum Dirs {
    Bin,     // Executable dir
    Cache,   // Might need in the future
    Config,  //
    Data,    //
    Home,    // Just $HOME
    Runtime, // For IPC socket
    State,   //
}

#[derive(Debug)]
pub enum DirError {
    DoesNotExist(PathBuf),
    InvalidPath(PathBuf),
    IoError(io::Error),
    MissingVar(String),
}

impl Display for DirError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DoesNotExist(p) => writeln!(f, "Directory does not exist: {p:?}"),
            Self::InvalidPath(p) => writeln!(f, "Invalid path: {p:?}"),
            Self::IoError(io_err) => writeln!(f, "I/O error: {io_err}"),
            Self::MissingVar(var) => writeln!(f, "Missing environment variable: {var}"),
        }
    }
}

impl Error for DirError {}

// Lil helper for get_dir function(s)
fn get_xdg_path(env_var: &str, default: impl FnOnce() -> PathBuf) -> PathBuf {
    env::var_os(env_var)
        .map(PathBuf::from)
        .and_then(|p| p.is_absolute().then_some(p))
        .unwrap_or_else(default)
}

/// Used for getting a directory directly
// TODO: might want to make this a builder
pub fn get_dir(dir: Dirs) -> Result<PathBuf, DirError> {
    let home_dir = match env::var_os("HOME").map(PathBuf::from) {
        Some(p) if p.is_absolute() => p,
        Some(p) => return Err(DirError::InvalidPath(p)),
        None => return Err(DirError::MissingVar("HOME".into())),
    };

    let base_dir_path = match dir {
        Dirs::Home => home_dir,
        Dirs::Bin => get_xdg_path("XDG_BIN_HOME", || home_dir.join(".local/bin")),
        Dirs::Cache => get_xdg_path("XDG_CACHE_HOME", || home_dir.join(".cache")),
        Dirs::Config => get_xdg_path("XDG_CONFIG_HOME", || home_dir.join(".config")),
        Dirs::Data => get_xdg_path("XDG_DATA_HOME", || home_dir.join(".local/share")),
        Dirs::Runtime => env::var_os("XDG_RUNTIME_DIR")
            .map(PathBuf::from)
            .and_then(|p| p.is_absolute().then_some(p))
            .ok_or_else(|| DirError::MissingVar("XDG_RUNTIME_DIR".into()))?,
        Dirs::State => get_xdg_path("XDG_STATE_HOME", || home_dir.join(".local/state")),
    };

    Ok(base_dir_path)
}

/// Used for getting a directory with a walrus directory at the end
pub fn get_app_dir(dir: Dirs) -> Result<PathBuf, DirError> {
    let path = get_dir(dir)?.join(APPNAME);
    if !path.exists() {
        return Err(DirError::DoesNotExist(path));
    }
    Ok(path)
}

/// Used for getting a directory directly
pub fn get_dir_with<P: AsRef<Path>>(dir: Dirs, append_dir: P) -> Result<PathBuf, DirError> {
    let path = get_dir(dir)?.join(append_dir);
    if !path.exists() {
        return Err(DirError::DoesNotExist(path));
    }
    Ok(path)
}

pub fn get_app_dir_with<P: AsRef<Path>>(dir: Dirs, append_dir: P) -> Result<PathBuf, DirError> {
    let path = get_dir(dir)?.join(APPNAME).join(append_dir);
    if !path.exists() {
        return Err(DirError::DoesNotExist(path));
    }
    Ok(path)
}

pub fn get_config_file<P: AsRef<Path>>(filename: P) -> Result<PathBuf, DirError> {
    let config_dir = match get_app_dir(Dirs::Config) {
        Ok(p) => p,
        Err(DirError::DoesNotExist(path)) => {
            fs::create_dir_all(&path).map_err(DirError::IoError)?;
            path
        }
        Err(e) => return Err(e),
    };
    if !config_dir.exists() {
        warn!("Config directory does not exist");
        fs::create_dir_all(&config_dir).map_err(DirError::IoError)?;
        info!("Created config directory at: {:?}", config_dir);
    }
    let config_file = config_dir.join(filename);
    if !config_file.exists() {
        warn!("Config file does not exist");
        File::create(&config_file).map_err(DirError::IoError)?;
        info!("Created config file at: {:?}", config_file);
    }
    Ok(config_file)
}

pub fn init_logger(log_level: LevelFilter) -> Result<(), Box<dyn Error>> {
    fn try_combined_logger(log_level: LevelFilter) -> Result<(), Box<dyn Error>> {
        let log_dir = match get_app_dir_with(Dirs::State, "logs") {
            Ok(p) => p,
            Err(DirError::DoesNotExist(path)) => {
                fs::create_dir_all(&path).map_err(DirError::IoError)?;
                path
            }
            Err(
                e @ (DirError::InvalidPath(_) | DirError::IoError(_) | DirError::MissingVar(_)),
            ) => {
                error!("Not logging to file: {e}");
                return Err(e.into());
            }
        };

        let time = OffsetDateTime::now_local().unwrap_or_else(|e| {
            error!("Failed to get local time offset: {e}");
            warn!("Falling back to UTC");
            OffsetDateTime::now_utc()
        });

        let log_name = time
            .replace_nanosecond(0)
            .expect("Error converting nanoseconds")
            .format(&well_known::Rfc3339)
            .expect("Invalid format")
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

    match try_combined_logger(log_level) {
        Ok(_) => Ok(()),
        Err(comb_err) => {
            match TermLogger::init(
                log_level,
                Config::default(),
                TerminalMode::Mixed,
                ColorChoice::Auto,
            ) {
                Ok(_) => {
                    error!("Failed to initialize WriteLogger: {}", comb_err);
                    Ok(())
                }
                Err(term_err) => Err(format!(
                    "Failed to initialise logging:\n- CombinedLogger: {}, TermLogger: {}",
                    comb_err, term_err
                )
                .into()),
            }
        }
    }
}

pub fn normalize_duration(base_duration: f64, res: Resolution, angle_degrees: f32) -> f64 {
    let width = f64::from(res.width);
    let height = f64::from(res.height);

    let theta: f64 = angle_degrees.to_radians().into();
    let distance_at_angle = (width * theta.cos().abs()) + (height * theta.sin().abs());
    debug!("DistanceAtAngle: {distance_at_angle}");
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

pub fn deserialize_flavour<'de, D>(d: D) -> Result<Option<Vec<TransitionFlavour>>, D::Error>
where
    D: Deserializer<'de>,
{
    let flavours: Option<Vec<String>> = Option::deserialize(d)?;

    match flavours {
        Some(flavours) => {
            let result: Result<Vec<TransitionFlavour>, D::Error> = flavours
                .into_iter()
                .map(|flavour| {
                    TransitionFlavour::from_str(&flavour.to_lowercase())
                        .map_err(serde::de::Error::custom)
                })
                .collect();
            result.map(Some)
        }
        None => Ok(None),
    }
}
