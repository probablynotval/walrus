use directories::{ProjectDirs, UserDirs};
use serde::{Deserialize, Serialize};
use std::{
    error::Error,
    fmt,
    fs::{self, File},
    path::PathBuf,
};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    #[serde(default)] // Unsure if this is needed
    pub general: Option<General>,
    #[serde(default)] // Unsure if this is needed
    pub transition: Option<Transition>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct General {
    pub interval: Option<u64>,
    pub path: Option<PathBuf>,
    pub shuffle: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct Transition {
    pub duration: Option<f32>,
    pub fill: Option<String>,
    pub filter: Option<String>,
    pub fps: Option<u32>,
    pub step: Option<u8>,
    pub resize: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            general: Some(General::default()),
            transition: Some(Transition::default()),
        }
    }
}

impl Default for General {
    fn default() -> Self {
        let path = UserDirs::new()
            .map(|user_dirs| user_dirs.picture_dir().unwrap().to_path_buf())
            .unwrap()
            .join("Wallpapers");

        General {
            interval: Some(300),
            path: Some(path),
            shuffle: Some(true),
        }
    }
}

impl Default for Transition {
    fn default() -> Self {
        Transition {
            duration: Some(0.75),
            fill: Some(String::from("000000")),
            filter: Some(String::from("Lanczos3")),
            fps: Some(60),
            step: Some(160),
            resize: Some(String::from("crop")),
        }
    }
}

impl fmt::Display for Config {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Current configuration")?;
        writeln!(f, "---------------------")?;
        writeln!(f, "{}", self.general.clone().unwrap_or_default())?;
        writeln!(f, "{}", self.transition.clone().unwrap_or_default())?;

        Ok(())
    }
}

impl fmt::Display for General {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "[General]")?;
        writeln!(
            f,
            "Interval: {}",
            self.interval
                .map(|x| x.to_string())
                .unwrap_or_else(|| "None".into())
        )?;
        writeln!(
            f,
            "Path: {}",
            self.path.as_deref().map(|x| x.display()).unwrap()
        )?;
        writeln!(
            f,
            "Shuffle: {}",
            self.shuffle
                .map(|x| x.to_string())
                .unwrap_or_else(|| "None".into())
        )
    }
}

impl fmt::Display for Transition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "[Transition]")?;
        writeln!(
            f,
            "Duration: {}",
            self.duration
                .map(|x| x.to_string())
                .unwrap_or_else(|| "None".into())
        )?;
        writeln!(
            f,
            "Fill: {}",
            self.fill.as_deref().unwrap_or_else(|| "None".into())
        )?;
        writeln!(
            f,
            "Filter: {}",
            self.filter.as_deref().unwrap_or_else(|| "None".into())
        )?;
        writeln!(
            f,
            "FPS: {}",
            self.fps
                .map(|x| x.to_string())
                .unwrap_or_else(|| "None".into())
        )?;
        writeln!(
            f,
            "Step: {}",
            self.step
                .map(|x| x.to_string())
                .unwrap_or_else(|| "None".into())
        )?;
        writeln!(
            f,
            "Resize: {}",
            self.resize.as_deref().unwrap_or_else(|| "None".into())
        )
    }
}

impl Config {
    pub fn from(path: &str) -> Result<Self, Box<dyn Error>> {
        if let Some(project_dirs) = ProjectDirs::from("qual", "org", "walrus") {
            let config_dir = project_dirs.config_dir();
            let walrus_dir_exists = match fs::metadata(config_dir) {
                Ok(metadata) => metadata.is_dir(),
                Err(_) => false,
            };
            if !walrus_dir_exists {
                fs::create_dir_all(config_dir)?;
            };

            let config_file = config_dir.join(path);
            let config_file_exists = match fs::metadata(&config_file) {
                Ok(metadata) => metadata.is_file(),
                Err(_) => false,
            };
            if !config_file_exists {
                File::create_new(&config_file).unwrap_or_else(|err| {
                    todo!("Handle error cases for creating config.toml: {err}")
                });
            };
            let config_raw = fs::read_to_string(&config_file)?;
            let config: Config = toml::from_str(&config_raw.as_str())
                .unwrap_or_else(|err| todo!("Handle error cases for config parsing: {err}"));
            Ok(config)
        } else {
            Err("Failed to create Config object".into())
        }
    }
}
