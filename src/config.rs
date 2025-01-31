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

use crate::{
    commands::Commands,
    error::ParseTransitionFlavourError,
    utils::{deserialize_flavour, get_config_file},
    wayland::WaylandHandle,
};

const DEFAULT_BEZIER: [f32; 4] = [0.4, 0.0, 0.6, 1.0];
const DEFAULT_DEBUG: &str = "info";
const DEFAULT_DURATION: f64 = 1.0;
const DEFAULT_DYNAMIC_DURATION: bool = true;
const DEFAULT_INTERVAL: u64 = 300;
const DEFAULT_FILL: &str = "000000";
const DEFAULT_FILTER: &str = "Lanczos3";
const DEFAULT_FLAVOUR: [TransitionFlavour; 4] = [
    TransitionFlavour::Wipe,
    TransitionFlavour::Wave,
    TransitionFlavour::Grow,
    TransitionFlavour::Outer,
];
const DEFAULT_RESIZE: &str = "crop";
const DEFAULT_SHUFFLE: bool = true;
const DEFAULT_STEP: u8 = 60;
const DEFAULT_SWW_PATH: &str = "/usr/bin/swww";
const DEFAULT_WALLPAPER_DIR: &str = "Wallpapers";
const DEFAULT_WAVE_SIZE: (i32, i32, i32, i32) = (55, 60, 45, 50);

const FALLBACK_FPS: u32 = 60;
const FALLBACK_RESOLUTION: Resolution = Resolution {
    width: 1920,
    height: 1080,
};

#[derive(Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct Config {
    pub general: Option<General>,
    pub transition: Option<Transition>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct General {
    pub debug: Option<String>,
    pub interval: Option<u64>,
    pub resolution: Option<Resolution>,
    pub shuffle: Option<bool>,
    pub swww_path: Option<String>,
    pub wallpaper_path: Option<PathBuf>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct Transition {
    pub bezier: Option<[f32; 4]>,
    pub duration: Option<f64>,
    pub dynamic_duration: Option<bool>,
    pub fill: Option<String>,
    pub filter: Option<String>,
    #[serde(deserialize_with = "deserialize_flavour")]
    pub flavour: Option<Vec<TransitionFlavour>>,
    pub fps: Option<u32>,
    pub resize: Option<String>,
    pub step: Option<u8>,
    pub wave_size: Option<(i32, i32, i32, i32)>,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq)]
pub struct Resolution {
    pub width: i32,
    pub height: i32,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum TransitionFlavour {
    Wipe,
    Wave,
    Grow,
    Outer,
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
            resolution: None,
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
            flavour: Some(DEFAULT_FLAVOUR.into()),
            fps: None,
            resize: Some(DEFAULT_RESIZE.into()),
            step: Some(DEFAULT_STEP),
            wave_size: Some(DEFAULT_WAVE_SIZE),
        }
    }
}

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
        let resolution = self.resolution.as_ref().unwrap();
        writeln!(
            f,
            "resolution = {{ width = {}, height = {} }}",
            resolution.width, resolution.height
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
        let flavour_str = self
            .flavour
            .clone()
            .unwrap_or(DEFAULT_FLAVOUR.into())
            .iter()
            .map(|f| f.to_string())
            .collect::<Vec<String>>()
            .join(", ");
        writeln!(f, "flavour = [{}]", flavour_str)?;
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

impl Config {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let path = get_config_file("config.toml");
        let config_raw = fs::read_to_string(&path)?;
        Ok(Self::from_str(&config_raw))
    }

    fn from_str(config_raw: &str) -> Self {
        let mut config: Self = toml::from_str(config_raw).unwrap_or_else(|e| {
            warn!("Syntax error in config: {e}\nFalling back to defaults...");
            Self::default()
        });

        let (fps, res) = match WaylandHandle::new() {
            Ok(mut wayland) => {
                let outputs = wayland.get_outputs();
                outputs.first().map_or_else(
                    || {
                        warn!("No monitors found");
                        (FALLBACK_FPS, FALLBACK_RESOLUTION)
                    },
                    |m| {
                        let fps = m.refresh_rate.round() as u32;
                        let res = Resolution {
                            width: m.resolution.0,
                            height: m.resolution.1,
                        };
                        (fps, res)
                    },
                )
            }
            Err(e) => {
                warn!("Failed to connect to Wayland: {e}");
                (FALLBACK_FPS, FALLBACK_RESOLUTION)
            }
        };

        if let Some(general) = &mut config.general {
            general.resolution.get_or_insert(res);
        } else {
            config.general = Some(General {
                resolution: Some(res),
                ..General::default()
            });
        }

        if let Some(transition) = &mut config.transition {
            transition.fps.get_or_insert(fps);
        } else {
            config.transition = Some(Transition {
                fps: Some(fps),
                ..Transition::default()
            })
        }

        config
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
            warn!("Watcher thread stopping; config hot reload stopped");
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

    pub fn resolution(&self) -> Resolution {
        self.general().resolution()
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
        self.interval.unwrap_or(DEFAULT_INTERVAL)
    }

    pub fn resolution(&self) -> Resolution {
        self.resolution.unwrap_or(FALLBACK_RESOLUTION)
    }

    pub fn shuffle(&self) -> bool {
        self.shuffle.unwrap_or(DEFAULT_SHUFFLE)
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
        self.bezier.unwrap_or(DEFAULT_BEZIER)
    }

    pub fn duration(&self) -> f64 {
        self.duration.unwrap_or(DEFAULT_DURATION)
    }

    pub fn dynamic_duration(&self) -> bool {
        self.dynamic_duration.unwrap_or(DEFAULT_DYNAMIC_DURATION)
    }

    pub fn fill(&self) -> String {
        self.fill.as_deref().unwrap_or(DEFAULT_FILL).into()
    }

    pub fn filter(&self) -> String {
        self.filter.as_deref().unwrap_or(DEFAULT_FILTER).into()
    }

    pub fn flavour(&self) -> Vec<TransitionFlavour> {
        self.flavour.clone().unwrap_or(DEFAULT_FLAVOUR.into())
    }

    pub fn fps(&self) -> u32 {
        self.fps.unwrap_or(FALLBACK_FPS)
    }

    pub fn resize(&self) -> String {
        self.resize.as_deref().unwrap_or(DEFAULT_RESIZE).into()
    }

    pub fn step(&self) -> u8 {
        self.step.unwrap_or(DEFAULT_STEP)
    }

    pub fn wave_size(&self) -> (i32, i32, i32, i32) {
        self.wave_size.unwrap_or(DEFAULT_WAVE_SIZE)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // NOTE: This test picks the monitor with the highest refresh rate. If more than one have the
    // highest refresh rate, the one with highest resolution will be picked.
    #[test]
    fn test_hierarchy() {
        // This test should assert that the correct values in the hierarchy are used:
        // User config --> Automatically inferred values (Wayland protocol) --> Fallback defaults

        let toml = r#"
            [general]
            resolution = { width = 69, height = 420 }

            [transition]
            fps = 42069 
        "#;

        let config = Config::from_str(toml);

        // 1. Assert that user config is used above all else
        assert_eq!(
            config.resolution(),
            Resolution {
                width: 69,
                height: 420
            }
        );
        assert_eq!(config.fps(), 42069);

        let config = Config::new().expect("Failed to initialise config");

        let mut wlhandle = WaylandHandle::new().expect("Failed to create handle");
        let outputs = wlhandle.get_outputs();

        // Pick the monitor with the highest refresh rate
        // If there is more than one monitor with the highest value pick the highest resolution
        let monitor = outputs
            .iter()
            .max_by(|a, b| {
                a.refresh_rate.total_cmp(&b.refresh_rate).then_with(|| {
                    let ap = a.resolution.0 as i64 * a.resolution.1 as i64;
                    let bp = b.resolution.0 as i64 * b.resolution.1 as i64;
                    ap.cmp(&bp)
                })
            })
            .expect("Monitor returned empty iterator (no monitor was found)");

        let wl_res = Resolution {
            width: monitor.resolution.0,
            height: monitor.resolution.1,
        };
        let wl_fps = monitor.refresh_rate.round() as u32;

        // 2. Assert that automatic values are used above fallback values
        assert_eq!(config.resolution(), wl_res);
        assert_eq!(config.fps(), wl_fps);

        // 3. After this fallback values would be used...
    }
}
