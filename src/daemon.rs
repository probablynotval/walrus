use crate::{
    commands::Commands,
    config::{Config, TransitionFlavour},
    utils::normalize_duration,
};
use log::{debug, error, info, trace};
use rand::{seq::SliceRandom, thread_rng, Rng};
use std::{
    env, fmt, fs,
    path::Path,
    process::Command,
    sync::mpsc::{self, Receiver},
    time::Duration,
};
use walkdir::WalkDir;

#[derive(Clone)]
pub struct Daemon {
    pub config: Config,
    pub index: usize,
    pub paused: bool,
    pub queue: Vec<String>,
}

impl Daemon {
    pub fn new(config: Config) -> Option<Self> {
        let directory = config
            .clone()
            .general
            .unwrap_or_default()
            .wallpaper_path
            .unwrap_or_default();
        let mut queue = Vec::new();
        for entry in WalkDir::new(directory).follow_links(true) {
            let entry = entry.unwrap();
            if entry.path().is_file() {
                if let Some(path) = entry.path().to_str() {
                    queue.push(path.to_string());
                }
            }
        }
        Some(Self {
            config,
            paused: false,
            queue,
            index: 0,
        })
    }

    pub fn run(&mut self, rx: Receiver<Commands>) {
        let config = self.config.clone();
        let general = config.clone().general.unwrap_or_default();
        let interval = general.interval.unwrap_or_default();
        let shuffle = general.shuffle.unwrap_or_default();

        if shuffle {
            trace!("Pre-shuffle\n{self}");
            self.shuffle_queue();
            trace!("Shuffled queue\n{self}");
        }
        self.current_wallpaper();

        let mut cont = true;
        while cont {
            let timeout = Duration::from_secs(interval);
            match rx.recv_timeout(timeout) {
                Ok(Commands::Config) => unreachable!(),
                Ok(Commands::Next) => {
                    debug!("Received Next command");
                    self.next_wallpaper();
                }
                Ok(Commands::Pause) => {
                    debug!("Received Pause command");
                    self.pause();
                }
                Ok(Commands::Previous) => {
                    debug!("Received Previous command");
                    self.previous_wallpaper();
                }
                Ok(Commands::Resume) => {
                    debug!("Received Resume command");
                    self.resume();
                }
                Ok(Commands::Shutdown) => {
                    debug!("Received Shutdown command");
                    if Path::new("/tmp/walrus.sock").exists() {
                        let _ = fs::remove_file("/tmp/walrus.sock");
                    }
                    cont = false;
                }
                Err(mpsc::RecvTimeoutError::Timeout) => {
                    debug!("Timeout: paused, not changing wallpapers");
                    if !self.paused {
                        debug!("Timeout: changing wallpapers...");
                        self.next_wallpaper();
                    }
                }
                Err(mpsc::RecvTimeoutError::Disconnected) => {
                    error!("Timeout: channel disconnected");
                    cont = false;
                }
            }
        }
    }

    fn current_wallpaper(&mut self) {
        if let Some(wallpaper) = self.queue.get(self.index) {
            info!("Setting wallpaper: {wallpaper}");
            self.set_wallpaper(wallpaper.into());
        }
    }

    fn get_transition(&mut self) {
        let transition = self.config.clone().transition.unwrap_or_default();
        let bezier = transition.bezier.unwrap_or_default();
        let duration = transition.duration.unwrap_or_default();
        let dynamic_duration = transition.dynamic_duration.unwrap_or_default();
        let flavour = transition.flavour.unwrap_or_default();
        let fps = transition.fps.unwrap_or_default();
        let step = transition.step.unwrap_or_default();
        let (wave_width_min, wave_width_max, wave_height_min, wave_height_max) =
            transition.wave_size.unwrap_or_default();

        env::set_var(
            "SWWW_TRANSITION_BEZIER",
            format!("{},{},{},{}", bezier[0], bezier[1], bezier[2], bezier[3]),
        );
        env::set_var("SWWW_TRANSITION_DURATION", duration.to_string());
        env::set_var("SWWW_TRANSITION_FPS", fps.to_string());
        env::set_var("SWWW_TRANSITION_STEP", step.to_string());

        let mut rng = rand::thread_rng();
        let flavour_rng = rng.gen_range(0..flavour.len());
        let flavour_str = flavour.get(flavour_rng).unwrap().as_str();
        let flavour_selection = TransitionFlavour::try_from(flavour_str).unwrap();
        env::set_var("SWWW_TRANSITION", flavour_str);
        debug!("Flavour: {flavour_str}");

        match flavour_selection {
            TransitionFlavour::Wipe | TransitionFlavour::Wave => {
                let angle_rng = rng.gen_range(0.0..360.0);
                trace!("Angle: {angle_rng}");
                if dynamic_duration {
                    let normalized_duration =
                        normalize_duration(duration, 2560.0, 1440.0, angle_rng);
                    env::set_var("SWWW_TRANSITION_DURATION", normalized_duration.to_string());
                    trace!("Dynamic duration: {normalized_duration}");
                }
                trace!("Duration: {duration}");
                env::set_var("SWWW_TRANSITION_ANGLE", angle_rng.to_string());

                if matches!(flavour_selection, TransitionFlavour::Wave) {
                    let width_wave_rng = rng.gen_range(wave_width_min..=wave_width_max);
                    let height_wave_rng = rng.gen_range(wave_height_min..=wave_height_max);
                    env::set_var(
                        "SWWW_TRANSITION_WAVE",
                        format!("{},{}", width_wave_rng, height_wave_rng),
                    );
                    env::set_var("SWWW_TRANSITION", "wave");
                }
            }
            TransitionFlavour::Grow | TransitionFlavour::Outer => {
                let x_position_rng = rng.gen::<f64>();
                let y_position_rng = rng.gen::<f64>();
                env::set_var(
                    "SWWW_TRANSITION_POS",
                    format!("{},{}", x_position_rng, y_position_rng),
                );
            }
        };
    }

    fn next_wallpaper(&mut self) {
        self.index = (self.index + 1) % self.queue.len();
        if let Some(wallpaper) = self.queue.get(self.index) {
            info!("Setting wallpaper: {wallpaper}");
            self.set_wallpaper(wallpaper.into());
        }
    }

    fn pause(&mut self) {
        self.paused = true;
    }

    fn previous_wallpaper(&mut self) {
        self.index = (self.index + self.queue.len() - 1) % self.queue.len();
        if let Some(wallpaper) = self.queue.get(self.index) {
            info!("Setting wallpaper: {wallpaper}");
            self.set_wallpaper(wallpaper.into());
        }
    }

    fn shuffle_queue(&mut self) {
        let mut rng = thread_rng();
        self.queue.shuffle(&mut rng);
        self.index = 0;
    }

    fn set_wallpaper(&mut self, path: String) {
        self.get_transition();

        let swww_path = self
            .config
            .clone()
            .general
            .unwrap_or_default()
            .swww_path
            .unwrap_or_default();

        let _ = Command::new(swww_path)
            .arg("img")
            .arg(path)
            .spawn()
            .unwrap()
            .wait();
    }

    fn resume(&mut self) {
        self.paused = false;
    }
}

impl fmt::Display for Daemon {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, wallpaper) in self.queue.iter().enumerate() {
            writeln!(f, "{} - {}", i, wallpaper)?;
        }
        Ok(())
    }
}
