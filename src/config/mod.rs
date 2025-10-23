use std::{
    cmp::Ordering,
    error::Error,
    fmt::{self, Display, Formatter},
    str::FromStr,
};

use serde::{Deserialize, Serialize};

pub use self::core::Config;

mod core;
mod defaults {
    use super::{Resolution, TransitionFlavour};

    pub(super) const DEFAULT_BEZIER: [f32; 4] = [0.4, 0.0, 0.6, 1.0];
    pub(super) const DEFAULT_DEBUG: &str = "info";
    pub(super) const DEFAULT_DURATION: f64 = 1.0;
    pub(super) const DEFAULT_DYNAMIC_DURATION: bool = true;
    pub(super) const DEFAULT_INTERVAL: u64 = 300;
    pub(super) const DEFAULT_FILL: &str = "000000";
    pub(super) const DEFAULT_FILTER: &str = "Lanczos3";
    pub(super) const DEFAULT_FLAVOUR: [TransitionFlavour; 4] = [
        TransitionFlavour::Wipe,
        TransitionFlavour::Wave,
        TransitionFlavour::Grow,
        TransitionFlavour::Outer,
    ];
    pub(super) const DEFAULT_RESIZE: &str = "crop";
    pub(super) const DEFAULT_SHUFFLE: bool = true;
    pub(super) const DEFAULT_STEP: u8 = 60;
    pub(super) const DEFAULT_SWW_PATH: &str = "/usr/bin/swww";
    pub(super) const DEFAULT_WALLPAPER_DIR: &str = "Wallpapers";
    pub(super) const DEFAULT_WAVE_SIZE: (u32, u32, u32, u32) = (70, 80, 35, 40);

    pub(super) const FALLBACK_FPS: u32 = 60;
    pub(super) const FALLBACK_RESOLUTION: Resolution = Resolution {
        width: 1920,
        height: 1080,
    };
}

#[derive(Debug)]
pub struct MonitorInfo {
    pub refresh_rate: f32,
    pub resolution: Resolution,
    pub id: u32,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq)]
pub struct Resolution {
    pub width: i32,
    pub height: i32,
}

pub struct BiggestArea<'a>(pub &'a Resolution);

impl Eq for BiggestArea<'_> {}

impl Ord for BiggestArea<'_> {
    fn cmp(&self, other: &Self) -> Ordering {
        let area_self = self.0.width as i64 * self.0.height as i64;
        let area_other = other.0.width as i64 * other.0.height as i64;
        area_self.cmp(&area_other)
    }
}

impl PartialEq for BiggestArea<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

impl PartialOrd for BiggestArea<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

pub struct HighestRefreshRate<'a>(pub &'a MonitorInfo);

impl Eq for HighestRefreshRate<'_> {}

impl Ord for HighestRefreshRate<'_> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.refresh_rate.total_cmp(&other.0.refresh_rate)
    }
}

impl PartialEq for HighestRefreshRate<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

impl PartialOrd for HighestRefreshRate<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

pub struct HighestResolution<'a>(pub &'a MonitorInfo);

impl Eq for HighestResolution<'_> {}

impl Ord for HighestResolution<'_> {
    fn cmp(&self, other: &Self) -> Ordering {
        BiggestArea(&self.0.resolution).cmp(&BiggestArea(&other.0.resolution))
    }
}

impl PartialEq for HighestResolution<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

impl PartialOrd for HighestResolution<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum TransitionFlavour {
    Wipe,
    Wave,
    Grow,
    Outer,
}

impl Display for TransitionFlavour {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Wipe => "wipe",
            Self::Wave => "wave",
            Self::Grow => "grow",
            Self::Outer => "outer",
        })
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

#[derive(Debug)]
pub enum ParseTransitionFlavourError {
    InvalidFlavour(String),
}

impl Display for ParseTransitionFlavourError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidFlavour(s) => writeln!(f, "Invalid transition type: {s}"),
        }
    }
}

impl Error for ParseTransitionFlavourError {}
