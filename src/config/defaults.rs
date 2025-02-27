use super::types::{Resolution, TransitionFlavour};

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
