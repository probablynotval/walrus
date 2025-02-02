use serde::{Deserialize, Serialize};

pub struct BiggestArea<'a>(pub &'a Resolution);

pub struct HighestRefreshRate<'a>(pub &'a MonitorInfo);

pub struct HighestResolution<'a>(pub &'a MonitorInfo);

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

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum TransitionFlavour {
    Wipe,
    Wave,
    Grow,
    Outer,
}

#[derive(Debug)]
pub enum ParseTransitionFlavourError {
    InvalidFlavour(String),
}
