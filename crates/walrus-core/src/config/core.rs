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

use notify::RecommendedWatcher;
use notify::Watcher;
use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;

use super::HighestRefreshRate;
use super::HighestResolution;
use super::Resolution;
use super::TransitionFlavour;
use super::defaults::*;
use crate::commands::Commands;
use crate::config::Bezier;
use crate::config::FilterMethod;
use crate::config::ResizeMethod;
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
                    tracing::error!("Error getting config file: {}", e);
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
            tracing::error!("Error parsing config: {e}");
            tracing::warn!("Falling back to default config...");
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
                        tracing::error!("No monitors found");
                        tracing::warn!("Falling back to default FPS and resolution");
                        (FALLBACK_FPS, FALLBACK_RESOLUTION)
                    },
                    |m| (m.refresh_rate.round() as u32, m.resolution),
                ),
            Err(e) => {
                tracing::warn!("Failed to connect to Wayland: {e}");
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
        tracing::debug!("Starting watcher...");
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
                tracing::debug!("File event: {event:?}");
                if event.kind.is_modify() || event.kind.is_remove() {
                    cmd_tx.send(Commands::Reload).unwrap();

                    if event.kind.is_remove() {
                        tracing::debug!("File removed, trying to re-establish watch");
                        thread::sleep(Duration::from_millis(200));
                        watcher.watch(abs_path.as_ref(), notify::RecursiveMode::NonRecursive)?;
                    }
                }
            }
            tracing::warn!("Watcher thread stopping; config hot reloading stopped");
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

    pub fn duration(&self) -> f64 {
        self.transition().duration()
    }

    pub fn dynamic_duration(&self) -> bool {
        self.transition().dynamic_duration()
    }

    pub fn fill(&self) -> String {
        self.transition().fill()
    }

    pub fn filter(&self) -> FilterMethod {
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

    pub fn resize(&self) -> ResizeMethod {
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
        match toml::to_string(self) {
            Ok(toml) => write!(f, "{toml}"),
            Err(e) => write!(f, "Error serializing config: {e}"),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default)]
pub(super) struct General {
    pub(super) interval: Option<u64>,
    pub(super) resolution: Option<Resolution>,
    pub(super) shuffle: Option<bool>,
    pub(super) swww_path: Option<String>,
    pub(super) wallpaper_path: Option<PathBuf>,
}

impl General {
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
            interval: Some(DEFAULT_INTERVAL),
            resolution: None,
            shuffle: Some(DEFAULT_SHUFFLE),
            swww_path: Some(DEFAULT_SWW_PATH.into()),
            wallpaper_path: Some(wallpaper_path),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default)]
pub(super) struct Transition {
    pub(super) bezier: Option<Bezier>,
    pub(super) duration: Option<f64>,
    pub(super) dynamic_duration: Option<bool>,
    pub(super) fill: Option<String>,
    #[serde(deserialize_with = "deserialize_filter")]
    pub(super) filter: Option<FilterMethod>,
    #[serde(deserialize_with = "deserialize_flavour")]
    pub(super) flavour: Option<Vec<TransitionFlavour>>,
    pub(super) fps: Option<u32>,
    #[serde(deserialize_with = "deserialize_resize")]
    pub(super) resize: Option<ResizeMethod>,
    pub(super) step: Option<u8>,
    pub(super) wave_size: Option<(u32, u32, u32, u32)>,
}

impl Transition {
    pub fn bezier(&self) -> Bezier {
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

    pub fn filter(&self) -> FilterMethod {
        self.filter.clone().unwrap_or(DEFAULT_FILTER)
    }

    pub fn flavour(&self) -> Vec<TransitionFlavour> {
        self.flavour.clone().unwrap_or(DEFAULT_FLAVOUR.into())
    }

    pub fn fps(&self) -> u32 {
        self.fps.unwrap_or(FALLBACK_FPS)
    }

    pub fn resize(&self) -> ResizeMethod {
        self.resize.clone().unwrap_or(DEFAULT_RESIZE)
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
            filter: Some(DEFAULT_FILTER),
            flavour: Some(DEFAULT_FLAVOUR.into()),
            fps: None,
            resize: Some(DEFAULT_RESIZE),
            step: Some(DEFAULT_STEP),
            wave_size: Some(DEFAULT_WAVE_SIZE),
        }
    }
}

fn deserialize_filter<'de, D>(d: D) -> Result<Option<FilterMethod>, D::Error>
where
    D: Deserializer<'de>,
{
    let method: Option<String> = Option::deserialize(d)?;

    match method {
        Some(method) => {
            let result: Result<FilterMethod, D::Error> =
                FilterMethod::from_str(&method.to_lowercase()).map_err(serde::de::Error::custom);
            result.map(Some)
        }
        None => Ok(None),
    }
}

fn deserialize_flavour<'de, D>(d: D) -> Result<Option<Vec<TransitionFlavour>>, D::Error>
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

fn deserialize_resize<'de, D>(d: D) -> Result<Option<ResizeMethod>, D::Error>
where
    D: Deserializer<'de>,
{
    let method: Option<String> = Option::deserialize(d)?;

    match method {
        Some(method) => {
            let result: Result<ResizeMethod, D::Error> =
                ResizeMethod::from_str(&method.to_lowercase()).map_err(serde::de::Error::custom);
            result.map(Some)
        }
        None => Ok(None),
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
