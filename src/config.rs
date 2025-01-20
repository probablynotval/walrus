use directories::UserDirs;
use log::{debug, warn, LevelFilter};
use notify::{RecommendedWatcher, Watcher};
use serde::{Deserialize, Serialize};
use std::{
    error::Error,
    fmt::{self, Display},
    fs,
    path::{Path, PathBuf},
    str::FromStr,
    sync::mpsc::{self, Sender},
    thread,
    time::Duration,
};

use crate::{commands::Commands, utils::get_config_file};

const DEFAULT_BEZIER: [f32; 4] = [0.4, 0.0, 0.6, 1.0];
const DEFAULT_DEBUG: &str = "info";
const DEFAULT_DURATION: f64 = 1.0;
const DEFAULT_DYNAMIC_DURATION: bool = true;
const DEFAULT_INTERVAL: u64 = 300;
const DEFAULT_FILL: &str = "000000";
const DEFAULT_FILTER: &str = "Lanczos3";
const DEFAULT_FLAVOUR: [&str; 4] = ["wipe", "wave", "grow", "outer"];
// Maybe try to figure out a way to get refresh rate and use that by default
const DEFAULT_FPS: u32 = 60;
const DEFAULT_RESIZE: &str = "crop";
const DEFAULT_SHUFFLE: bool = true;
const DEFAULT_STEP: u8 = 60;
const DEFAULT_SWW_PATH: &str = "/usr/bin/swww";
const DEFAULT_WALLPAPER_DIR: &str = "Wallpapers";
const DEFAULT_WAVE_SIZE: (i32, i32, i32, i32) = (55, 60, 45, 50);

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
            .map(|user_dirs| {
                user_dirs
                    .picture_dir()
                    .expect("Failed to get Picture dir")
                    .to_path_buf()
            })
            .unwrap()
            .join(DEFAULT_WALLPAPER_DIR);

        General {
            debug: Some(DEFAULT_DEBUG.into()),
            interval: Some(DEFAULT_INTERVAL),
            shuffle: Some(DEFAULT_SHUFFLE),
            swww_path: Some(DEFAULT_SWW_PATH.into()),
            wallpaper_path: Some(wallpaper_path),
        }
    }
}

impl Default for Transition {
    fn default() -> Self {
        Transition {
            bezier: Some(DEFAULT_BEZIER),
            duration: Some(DEFAULT_DURATION),
            dynamic_duration: Some(DEFAULT_DYNAMIC_DURATION),
            fill: Some(DEFAULT_FILL.into()),
            filter: Some(DEFAULT_FILL.into()),
            flavour: Some(
                DEFAULT_FLAVOUR
                    .into_iter()
                    .map(|str| str.to_string())
                    .collect(),
            ),
            fps: Some(DEFAULT_FPS),
            resize: Some(DEFAULT_RESIZE.into()),
            step: Some(DEFAULT_STEP),
            wave_size: Some(DEFAULT_WAVE_SIZE),
        }
    }
}

// FIX: I think there could be a bug with dereferncing here, so if things seem right but printing
// the config seems to tell you something else, it's probably something wrong in here
impl Display for Config {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "\nCurrent configuration")?;
        writeln!(f, "---------------------")?;
        writeln!(f, "{}", self.general.clone().unwrap_or_default())?;
        writeln!(f, "{}", self.transition.clone().unwrap_or_default())?;
        Ok(())
    }
}

impl Display for General {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "[General]")?;
        writeln!(f, "debug = {}", self.debug.as_deref().unwrap_or("None"))?;
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
            self.shuffle.map(|x| x.to_string()).unwrap_or("None".into())
        )?;
        writeln!(
            f,
            "swww_path = {}",
            self.swww_path.as_deref().unwrap_or("None")
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
        writeln!(f, "fill = {}", self.fill.as_deref().unwrap_or("None"))?;
        writeln!(f, "filter = {}", self.filter.as_deref().unwrap_or("None"))?;
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
        writeln!(f, "resize = {}", self.resize.as_deref().unwrap_or("None"))?;
        let wave_size = self.wave_size.unwrap();
        writeln!(
            f,
            "wave_size = [{}, {}, {}, {}]",
            wave_size.0, wave_size.1, wave_size.2, wave_size.3,
        )
    }
}

impl Config {
    pub fn new(optpath: Option<&str>) -> Result<Self, Box<dyn Error>> {
        let path = get_config_file("config.toml");
        let config_raw = match optpath {
            Some(p) => fs::read_to_string(p)?,
            None => fs::read_to_string(&path)?,
        };
        let config: Self = toml::from_str(config_raw.as_str()).unwrap_or_else(|e| {
            warn!("Syntax error in config: {e}\nFalling back to defaults...");
            Self::default()
        });
        Ok(config)
    }

    pub fn watch<P: AsRef<Path>>(path: P, cmd_tx: Sender<Commands>) -> notify::Result<()> {
        debug!("Starting watcher...");
        let (tx, rx) = mpsc::channel();
        let mut watcher = RecommendedWatcher::new(
            tx,
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
                            cmd_tx.send(Commands::Reload).unwrap();

                            if event.kind.is_remove() {
                                debug!("File removed, trying to re-establish watch");
                                thread::sleep(Duration::from_millis(200));
                                watcher.watch(
                                    abs_path.as_ref(),
                                    notify::RecursiveMode::NonRecursive,
                                )?;
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

    pub fn general(&self) -> General {
        self.general.clone().unwrap_or_default()
    }

    pub fn transition(&self) -> Transition {
        self.transition.clone().unwrap_or_default()
    }

    pub fn bezier(&self) -> [f32; 4] {
        self.transition().bezier()
    }

    pub fn debug(&self) -> LevelFilter {
        self.general().debug()
    }

    pub fn duration(&self) -> f64 {
        self.transition().duration()
    }

    pub fn dynamic_duration(&self) -> bool {
        self.transition().dynamic_duration()
    }

    pub fn fill(&self) -> String {
        self.transition().fill()
    }

    pub fn filter(&self) -> String {
        self.transition().filter()
    }

    pub fn flavour(&self) -> Vec<TransitionFlavour> {
        self.transition().flavour()
    }

    pub fn fps(&self) -> u32 {
        self.transition().fps()
    }

    pub fn interval(&self) -> u64 {
        self.general().interval()
    }

    pub fn resize(&self) -> String {
        self.transition().resize()
    }

    pub fn shuffle(&self) -> bool {
        self.general().shuffle()
    }

    pub fn step(&self) -> u8 {
        self.transition().step()
    }

    pub fn swww_path(&self) -> String {
        self.general().swww_path()
    }

    pub fn wallpaper_path(&self) -> PathBuf {
        self.general().wallpaper_path()
    }

    pub fn wave_size(&self) -> (i32, i32, i32, i32) {
        self.transition().wave_size()
    }
}

impl General {
    pub fn debug(&self) -> LevelFilter {
        match LevelFilter::from_str(
            self.debug
                .as_deref()
                .unwrap_or(DEFAULT_DEBUG)
                .to_lowercase()
                .as_str(),
        ) {
            Ok(dbglvl) => dbglvl,
            Err(_) => {
                warn!("Unknown debug option in config, falling back to default.");
                LevelFilter::Info
            }
        }
    }

    pub fn interval(&self) -> u64 {
        self.interval.unwrap_or_default()
    }

    pub fn shuffle(&self) -> bool {
        self.shuffle.unwrap_or_default()
    }

    pub fn swww_path(&self) -> String {
        self.swww_path.as_deref().unwrap_or(DEFAULT_SWW_PATH).into()
    }

    pub fn wallpaper_path(&self) -> PathBuf {
        self.wallpaper_path.clone().unwrap_or_default()
    }
}

impl Transition {
    pub fn bezier(&self) -> [f32; 4] {
        self.bezier.unwrap_or_default()
    }

    pub fn duration(&self) -> f64 {
        self.duration.unwrap_or_default()
    }

    pub fn dynamic_duration(&self) -> bool {
        self.dynamic_duration.unwrap_or_default()
    }

    pub fn fill(&self) -> String {
        self.fill.as_deref().unwrap_or(DEFAULT_FILL).into()
    }

    pub fn filter(&self) -> String {
        self.filter.as_deref().unwrap_or(DEFAULT_FILTER).into()
    }

    pub fn flavour(&self) -> Vec<TransitionFlavour> {
        let fvec = self.flavour.clone().unwrap_or_default();
        fvec.into_iter()
            .filter_map(|s| match TransitionFlavour::from_str(s.as_str()) {
                Ok(flavour) => Some(flavour),
                Err(e) => {
                    warn!("Invalid transition type: '{s}': {e}");
                    None
                }
            })
            .collect()
    }

    pub fn fps(&self) -> u32 {
        self.fps.unwrap_or_default()
    }

    pub fn resize(&self) -> String {
        self.resize.as_deref().unwrap_or(DEFAULT_RESIZE).into()
    }

    pub fn step(&self) -> u8 {
        self.step.unwrap_or_default()
    }

    pub fn wave_size(&self) -> (i32, i32, i32, i32) {
        self.wave_size.unwrap_or_default()
    }
}

#[derive(Debug)]
pub enum ParseTransitionFlavourError {
    InvalidFlavour(String),
}

impl Display for ParseTransitionFlavourError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidFlavour(s) => write!(f, "Invalid transition type: {}", s),
        }
    }
}

impl Error for ParseTransitionFlavourError {}

#[derive(Clone, Debug)]
pub enum TransitionFlavour {
    Wipe,
    Wave,
    Grow,
    Outer,
}

impl FromStr for TransitionFlavour {
    type Err = ParseTransitionFlavourError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "wipe" => Ok(Self::Wipe),
            "wave" => Ok(Self::Wave),
            "grow" => Ok(Self::Grow),
            "outer" => Ok(Self::Outer),
            _ => Err(Self::Err::InvalidFlavour(s.to_string())),
        }
    }
}

impl Display for TransitionFlavour {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Wipe => "wipe",
            Self::Wave => "wave",
            Self::Grow => "grow",
            Self::Outer => "outer",
        })
    }
}
