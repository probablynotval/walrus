use std::error::Error;
use std::fmt;
use std::fmt::Display;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::mpsc;
use std::sync::mpsc::Sender;
use std::thread;
use std::time::Duration;

use log::LevelFilter;
use log::debug;
use log::error;
use log::warn;
use notify::RecommendedWatcher;
use notify::Watcher;
use serde::Deserialize;
use serde::Serialize;

use super::HighestRefreshRate;
use super::HighestResolution;
use super::Resolution;
use super::TransitionFlavour;
use super::defaults::*;
use crate::commands::Commands;
use crate::utils;
use crate::utils::DirError;
use crate::utils::Dirs;
use crate::wayland::WaylandHandle;

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct Config {
    pub(super) general: Option<General>,
    pub(super) transition: Option<Transition>,
}

impl Config {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let path = match utils::get_config_file("config.toml") {
            Ok(p) => p,
            Err(e) => match e {
                DirError::InvalidPath(_) | DirError::IoError(_) | DirError::MissingVar(_) => {
                    error!("Error getting config file: {}", e);
                    return Err(e.into());
                }
                DirError::DoesNotExist(_) => unreachable!(),
            },
        };
        let config_raw = fs::read_to_string(&path)?;
        Ok(Self::from_raw(&config_raw))
    }

    fn from_raw(config_raw: &str) -> Self {
        let mut config: Self = toml::from_str(config_raw).unwrap_or_else(|e| {
            error!("Error parsing config: {e}");
            warn!("Falling back to default config...");
            Self::default()
        });

        let (fps, res) = match WaylandHandle::new() {
            Ok(mut wayland) => wayland
                .get_outputs()
                .iter()
                .max_by(|a, b| {
                    HighestRefreshRate(a)
                        .cmp(&HighestRefreshRate(b))
                        .then_with(|| HighestResolution(a).cmp(&HighestResolution(b)))
                })
                .map_or_else(
                    || {
                        error!("No monitors found");
                        warn!("Falling back to default FPS and resolution");
                        (FALLBACK_FPS, FALLBACK_RESOLUTION)
                    },
                    |m| (m.refresh_rate.round() as u32, m.resolution),
                ),
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
                let event = event_res?;
                debug!("File event: {event:?}");
                if event.kind.is_modify() || event.kind.is_remove() {
                    cmd_tx.send(Commands::Reload).unwrap();

                    if event.kind.is_remove() {
                        debug!("File removed, trying to re-establish watch");
                        thread::sleep(Duration::from_millis(200));
                        watcher.watch(abs_path.as_ref(), notify::RecursiveMode::NonRecursive)?;
                    }
                }
            }
            warn!("Watcher thread stopping; config hot reloading stopped");
            Ok(())
        });

        Ok(())
    }
}

impl Config {
    fn general(&self) -> General {
        self.general.clone().unwrap_or_default()
    }

    fn transition(&self) -> Transition {
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

    pub fn wave_size(&self) -> (u32, u32, u32, u32) {
        self.transition().wave_size()
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

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default)]
pub(super) struct General {
    pub(super) debug: Option<String>,
    pub(super) interval: Option<u64>,
    pub(super) resolution: Option<Resolution>,
    pub(super) shuffle: Option<bool>,
    pub(super) swww_path: Option<String>,
    pub(super) wallpaper_path: Option<PathBuf>,
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

impl Default for General {
    fn default() -> Self {
        let wallpaper_path = utils::get_dir_with(Dirs::Home, "Pictures")
            .expect("Failed to get Pictures directory")
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

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default)]
pub(super) struct Transition {
    pub(super) bezier: Option<[f32; 4]>,
    pub(super) duration: Option<f64>,
    pub(super) dynamic_duration: Option<bool>,
    pub(super) fill: Option<String>,
    pub(super) filter: Option<String>,
    #[serde(deserialize_with = "utils::deserialize_flavour")]
    pub(super) flavour: Option<Vec<TransitionFlavour>>,
    pub(super) fps: Option<u32>,
    pub(super) resize: Option<String>,
    pub(super) step: Option<u8>,
    pub(super) wave_size: Option<(u32, u32, u32, u32)>,
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

    pub fn wave_size(&self) -> (u32, u32, u32, u32) {
        self.wave_size.unwrap_or(DEFAULT_WAVE_SIZE)
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
        writeln!(f, "flavour = [{flavour_str}]")?;
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

        let config = Config::from_raw(toml);

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
                HighestRefreshRate(a)
                    .cmp(&HighestRefreshRate(b))
                    .then_with(|| HighestResolution(a).cmp(&HighestResolution(b)))
            })
            .expect("Monitor returned empty iterator (no monitor was found)");

        let wl_res = monitor.resolution;
        let wl_fps = monitor.refresh_rate.round() as u32;

        // 2. Assert that automatic values are used above fallback values
        assert_eq!(config.resolution(), wl_res);
        assert_eq!(config.fps(), wl_fps);

        // 3. After this fallback values would be used...
    }
}
