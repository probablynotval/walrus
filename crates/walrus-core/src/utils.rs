use std::env;
use std::error::Error;
use std::fmt;
use std::fmt::Display;
use std::fs;
use std::fs::File;
use std::io;
use std::path::Path;
use std::path::PathBuf;

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
        tracing::warn!("Config directory does not exist");
        fs::create_dir_all(&config_dir).map_err(DirError::IoError)?;
        tracing::info!("Created config directory at: {:?}", config_dir);
    }
    let config_file = config_dir.join(filename);
    if !config_file.exists() {
        tracing::warn!("Config file does not exist");
        File::create(&config_file).map_err(DirError::IoError)?;
        tracing::info!("Created config file at: {:?}", config_file);
    }
    Ok(config_file)
}
