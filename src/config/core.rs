use std::{
    error::Error,
    fs,
    path::{Path, PathBuf},
    sync::mpsc::{self, Sender},
    thread,
    time::Duration,
};

use log::{debug, error, warn};
use notify::{RecommendedWatcher, Watcher};
use serde::{Deserialize, Serialize};

use super::{
    defaults::*,
    types::{HighestRefreshRate, HighestResolution, Resolution, TransitionFlavour},
};
use crate::{
    commands::Commands,
    utils::{self, DirError},
    wayland::WaylandHandle,
};

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct Config {
    pub(super) general: Option<General>,
    pub(super) transition: Option<Transition>,
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
