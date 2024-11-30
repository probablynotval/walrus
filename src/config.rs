use directories::{ProjectDirs, UserDirs};
use log::{warn, LevelFilter};
use serde::{Deserialize, Serialize};
use std::{
    error::Error,
    fmt::{self, Display},
    fs,
    path::PathBuf,
};

#[derive(Clone, Debug)]
pub enum TransitionFlavour {
    Wipe,
    Wave,
    Grow,
    Outer,
}

impl TryFrom<&str> for TransitionFlavour {
    type Error = &'static str;

    fn try_from(flavour: &str) -> Result<Self, Self::Error> {
        match flavour.to_lowercase().as_str() {
            "wipe" => Ok(Self::Wipe),
            "wave" => Ok(Self::Wave),
            "grow" => Ok(Self::Grow),
            "outer" => Ok(Self::Outer),
            &_ => Err("Transition type does not exist"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct Config {
    pub general: Option<General>,
    pub transition: Option<Transition>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct General {
    pub debug: Option<String>,
    pub interval: Option<u64>,
    pub shuffle: Option<bool>,
    pub swww_path: Option<String>,
    pub wallpaper_path: Option<PathBuf>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct Transition {
    pub bezier: Option<[f32; 4]>,
    pub duration: Option<f64>,
    pub dynamic_duration: Option<bool>,
    pub fill: Option<String>,
    pub filter: Option<String>,
    pub flavour: Option<Vec<String>>,
    pub fps: Option<u32>,
    pub resize: Option<String>,
    pub step: Option<u8>,
    pub wave_size: Option<(i32, i32, i32, i32)>,
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
        let wallpaper_path = UserDirs::new()
            .map(|user_dirs| user_dirs.picture_dir().unwrap().to_path_buf())
            .unwrap()
            .join("Wallpapers");

        General {
            debug: Some(String::from("info")),
            interval: Some(300),
            shuffle: Some(true),
            swww_path: Some(String::from("/usr/bin/swww")),
            wallpaper_path: Some(wallpaper_path),
        }
    }
}

impl Default for Transition {
    fn default() -> Self {
        Transition {
            bezier: Some([0.40, 0.0, 0.6, 1.0]),
            duration: Some(1.0),
            dynamic_duration: Some(true),
            fill: Some(String::from("000000")),
            filter: Some(String::from("Lanczos3")),
            flavour: Some(vec![
                "wipe".into(),
                "wave".into(),
                "grow".into(),
                "outer".into(),
            ]),
            fps: Some(60),
            resize: Some(String::from("crop")),
            step: Some(60),
            wave_size: Some((55, 60, 45, 50)),
        }
    }
}

impl Display for Config {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "")?;
        writeln!(f, "Current configuration")?;
        writeln!(f, "---------------------")?;
        writeln!(f, "{}", self.general.clone().unwrap_or_default())?;
        writeln!(f, "{}", self.transition.clone().unwrap_or_default())?;
        Ok(())
    }
}

impl Display for General {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "[General]")?;
        writeln!(
            f,
            "debug = {}",
            self.debug.as_deref().unwrap_or_else(|| "None".into())
        )?;
        writeln!(
            f,
            "interval = {}",
            self.interval
                .map(|x| x.to_string())
                .unwrap_or_else(|| "None".into())
        )?;
        writeln!(
            f,
            "shuffle = {}",
            self.shuffle
                .map(|x| x.to_string())
                .unwrap_or_else(|| "None".into())
        )?;
        writeln!(
            f,
            "swww_path = {}",
            self.swww_path.as_deref().unwrap_or_else(|| "None".into())
        )?;
        writeln!(
            f,
            "wallpaper_path = {}",
            self.wallpaper_path.as_deref().map(|x| x.display()).unwrap()
        )
    }
}

impl Display for Transition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "[Transition]")?;
        let bezier = self.bezier.unwrap();
        writeln!(
            f,
            "bezier = [{}, {}, {}, {}]",
            bezier[0], bezier[1], bezier[2], bezier[3]
        )?;
        writeln!(
            f,
            "duration = {}",
            self.duration
                .map(|x| x.to_string())
                .unwrap_or_else(|| "None".into())
        )?;
        writeln!(
            f,
            "dynamic_duration = {}",
            self.dynamic_duration
                .map(|x| x.to_string())
                .unwrap_or_else(|| "None".into())
        )?;
        writeln!(
            f,
            "fill = {}",
            self.fill.as_deref().unwrap_or_else(|| "None".into())
        )?;
        writeln!(
            f,
            "filter = {}",
            self.filter.as_deref().unwrap_or_else(|| "None".into())
        )?;
        if let Some(flavour) = &self.flavour {
            let flavours = flavour.join(", ");
            writeln!(f, "flavour = [{}]", flavours)?;
        }
        writeln!(
            f,
            "fps = {}",
            self.fps
                .map(|x| x.to_string())
                .unwrap_or_else(|| "None".into())
        )?;
        writeln!(
            f,
            "step = {}",
            self.step
                .map(|x| x.to_string())
                .unwrap_or_else(|| "None".into())
        )?;
        writeln!(
            f,
            "resize = {}",
            self.resize.as_deref().unwrap_or_else(|| "None".into())
        )?;
        let wave_size = self.wave_size.unwrap();
        writeln!(
            f,
            "wave_size = [{}, {}, {}, {}]",
            wave_size.0, wave_size.1, wave_size.2, wave_size.3,
        )
    }
}

impl Config {
    pub fn from(config_file: &str) -> Result<Self, Box<dyn Error>> {
        if let Some(project_dirs) = ProjectDirs::from("qual", "org", "walrus") {
            let config_dir = project_dirs.config_dir();
            let walrus_dir_exists = match fs::metadata(config_dir) {
                Ok(metadata) => metadata.is_dir(),
                Err(_) => false,
            };
            if !walrus_dir_exists {
                fs::create_dir_all(config_dir)?;
            };

            let config_file = config_dir.join(config_file);
            let config_raw = fs::read_to_string(&config_file)?;
            let config: Self = toml::from_str(&config_raw.as_str()).unwrap_or_else(|e| {
                warn!("Syntax error in config: {e}\nFalling back to defaults...");
                Self::default()
            });
            Ok(config)
        } else {
            Err("Failed to create Config object".into())
        }
    }

    pub fn get_debug_level(&self) -> LevelFilter {
        return match self
            .clone()
            .general
            .unwrap_or_default()
            .debug
            .unwrap_or_default()
            .to_lowercase()
            .as_str()
        {
            "off" | "0" => LevelFilter::Off,
            "error" | "1" => LevelFilter::Error,
            "warn" | "2" => LevelFilter::Warn,
            "info" | "3" => LevelFilter::Info,
            "debug" | "4" => LevelFilter::Debug,
            "trace" | "5" => LevelFilter::Trace,
            _ => {
                warn!("Unknown debug option in config, falling back to default.");
                LevelFilter::Info
            }
        };
    }
}
