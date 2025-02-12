use std::{
    cmp::Ordering,
    error::Error,
    fmt::{self, Display, Formatter},
    path::PathBuf,
    str::FromStr,
};

use log::{warn, LevelFilter};

use super::{
    core::{Config, General, Transition},
    defaults::*,
    types::{
        BiggestArea, HighestRefreshRate, HighestResolution, ParseTransitionFlavourError,
        Resolution, TransitionFlavour,
    },
};
use crate::utils::{self, Dirs};

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

impl Display for ParseTransitionFlavourError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidFlavour(s) => writeln!(f, "Invalid transition type: {}", s),
        }
    }
}

impl Error for ParseTransitionFlavourError {}

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
