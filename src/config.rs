use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::{
    error::Error,
    fs::{self, File},
    io,
};
use toml;

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub general: Option<General>,
    pub transition: Option<Transition>,
}

#[derive(Serialize, Deserialize)]
pub struct General {
    pub path: Option<String>,
    pub interval: Option<usize>,
    pub shuffle: Option<bool>,
}

#[derive(Serialize, Deserialize)]
pub struct Transition {
    pub fps: Option<usize>,
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
            let config_content = fs::read_to_string(&config_file)?;
            let config: Config = toml::from_str(&config_content.as_str())
                .unwrap_or_else(|err| todo!("Handle error cases for config parsing: {err}"));
            Ok(config)
        } else {
            Err("Failed to create Config object".into())
        }
    }
}
