use log::{debug, error, info, warn};
use rand::{rngs::SmallRng, seq::SliceRandom, thread_rng, Rng, SeedableRng};
use std::{
    env,
    fmt::{self, Display},
    fs,
    path::{Path, PathBuf},
    process::Command,
    sync::mpsc::{self, Receiver},
    time::Duration,
};
use walkdir::WalkDir;

use crate::{
    commands::Commands,
    config::{Config, TransitionFlavour},
    ipc::send_ipc_command,
    utils::{decrement_index, increment_index, normalize_duration, SCREENWH, SOCKET_PATH},
};

pub struct Daemon {
    pub config: Config,
    pub index: usize,
    pub paused: bool,
    pub queue: Vec<String>,
    rng: SmallRng,
    angle: Option<f64>,
}

impl Daemon {
    pub fn new(config: Config) -> Option<Self> {
        let directory = config.general().wallpaper_path();
        let walker = WalkDir::new(directory);
        let queue: Vec<String> = walker
            .follow_links(true)
            .into_iter()
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.path().is_file())
            .map(|entry| entry.path().to_string_lossy().to_string())
            .collect();
        debug!("Starting with Config: {}", config);
        Some(Self {
            angle: None,
            config,
            paused: false,
            queue,
            index: 0,
            rng: SmallRng::from_entropy(),
        })
    }

    pub fn run(&mut self, rx: Receiver<Commands>) {
        if self.config.shuffle() {
            self.shuffle_queue();
        } else {
            self.queue.sort();
        }
        debug!("{:#?}", self.queue);
        self.current_wallpaper();

        let mut cont = true;
        while cont {
            let interval = self.config.interval();
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
                Ok(Commands::Reload) => {
                    self.reload_config();
                }
                Ok(Commands::Shutdown) => {
                    debug!("Received Shutdown command");
                    if Path::new(SOCKET_PATH).exists() {
                        let _ = fs::remove_file(SOCKET_PATH);
                    }
                    cont = false;
                }
                Err(mpsc::RecvTimeoutError::Timeout) => {
                    if !self.paused {
                        debug!("Timeout: changing wallpapers...");
                        self.next_wallpaper();
                        continue;
                    }
                    debug!("Timeout: paused, not changing wallpapers");
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
        let bezier = self.config.bezier();
        env::set_var(
            "SWWW_TRANSITION_BEZIER",
            format!("{},{},{},{}", bezier[0], bezier[1], bezier[2], bezier[3]),
        );

        let duration = self.config.duration();
        debug!("Duration: {duration}");

        let fps = self.config.fps();
        env::set_var("SWWW_TRANSITION_FPS", fps.to_string());

        let step = self.config.step();
        env::set_var("SWWW_TRANSITION_STEP", step.to_string());

        let flavour = self.config.flavour();
        let flavour_rng = self.rng.gen_range(0..flavour.len());
        let flavour_selection = flavour.get(flavour_rng).unwrap();
        env::set_var("SWWW_TRANSITION", flavour_selection.to_string());
        debug!("Flavour: {}", flavour_selection.to_string());

        match flavour_selection {
            TransitionFlavour::Wipe => {
                let angle = self.rng.gen_range(0.0..360.0);
                self.angle = Some(angle);
                debug!("Angle: {angle}");
                env::set_var("SWWW_TRANSITION_ANGLE", angle.to_string());

                if self.config.dynamic_duration() {
                    // FIX: so there's two approaches to getting the correct screen dimensions.
                    // 1. try to derive this automatically, although it might not be robust and
                    //    has the potential to be complicated?
                    // 2. just make another configuration option and be done with it lol
                    let normalized_duration =
                        normalize_duration(duration, SCREENWH.0, SCREENWH.1, angle);
                    debug!("Dynamic duration: {normalized_duration}");
                    env::set_var("SWWW_TRANSITION_DURATION", normalized_duration.to_string());
                } else {
                    env::set_var(
                        "SWWW_TRANSITION_DURATION",
                        self.config.duration().to_string(),
                    );
                }
            }
            TransitionFlavour::Wave => {
                let angle = self.rng.gen_range(0.0..360.0);
                debug!("Angle: {angle}");
                let angle_string = ((360.0 + angle - 90.0) % 360.0).to_string();
                debug!("AngleString: {angle_string}");
                env::set_var("SWWW_TRANSITION_ANGLE", angle_string);

                if self.config.dynamic_duration() {
                    let normalized_duration =
                        normalize_duration(duration, SCREENWH.0, SCREENWH.1, angle);
                    debug!("Dynamic duration: {normalized_duration}");
                    env::set_var("SWWW_TRANSITION_DURATION", normalized_duration.to_string());
                } else {
                    env::set_var(
                        "SWWW_TRANSITION_DURATION",
                        self.config.duration().to_string(),
                    );
                }

                let (wave_width_min, wave_width_max, wave_height_min, wave_height_max) =
                    self.config.wave_size();
                let width_wave_rng = self.rng.gen_range(wave_width_min..=wave_width_max);
                let height_wave_rng = self.rng.gen_range(wave_height_min..=wave_height_max);
                env::set_var(
                    "SWWW_TRANSITION_WAVE",
                    format!("{},{}", width_wave_rng, height_wave_rng),
                );
            }
            TransitionFlavour::Grow | TransitionFlavour::Outer => {
                env::set_var("SWWW_TRANSITION_DURATION", duration.to_string());
                let x_position_rng = self.rng.gen::<f64>();
                let y_position_rng = self.rng.gen::<f64>();
                env::set_var(
                    "SWWW_TRANSITION_POS",
                    format!("{},{}", x_position_rng, y_position_rng),
                );
            }
        };
    }

    fn update_index(&mut self, increment: bool) {
        let mut attempts = 0;

        while attempts < self.queue.len() {
            self.index = match increment {
                true => increment_index(self.index, self.queue.len()),
                false => decrement_index(self.index, self.queue.len()),
            };
            if let Some(wallpaper) = self.queue.get(self.index) {
                let path = PathBuf::from(wallpaper);
                if !path.exists() {
                    warn!(
                        "File not found, removing from queue: {}",
                        path.to_str().unwrap_or_default()
                    );
                    self.queue.remove(self.index);
                    self.index -= 1;
                    continue;
                }
                info!("Setting wallpaper: {wallpaper}");
                self.set_wallpaper(wallpaper.into());
                return;
            }
            attempts += 1
        }

        error!("No valid path found in queue, shutting down");
        let _ = send_ipc_command(Commands::Shutdown);
    }

    fn next_wallpaper(&mut self) {
        self.update_index(true);
    }

    fn pause(&mut self) {
        self.paused = true;
    }

    fn previous_wallpaper(&mut self) {
        self.update_index(false);
    }

    fn shuffle_queue(&mut self) {
        let mut rng = thread_rng();
        self.queue.shuffle(&mut rng);
        self.index = 0;
    }

    fn set_wallpaper(&mut self, path: String) {
        self.get_transition();

        let _ = Command::new(self.config.swww_path())
            .arg("img")
            .arg(path)
            .spawn()
            .unwrap()
            .wait();
    }

    fn reload_config(&mut self) {
        info!("Reloading config...");
        debug!("Old config: {:#?}", self.config);
        let config = Config::new(None).unwrap_or_default();
        self.config = config;
        debug!("New config: {:#?}", self.config);
        log::set_max_level(self.config.debug());
    }

    fn resume(&mut self) {
        self.paused = false;
    }
}

impl Display for Daemon {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, wallpaper) in self.queue.iter().enumerate() {
            writeln!(f, "{} - {}", i, wallpaper)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::{error::Error, fs::File, io::Write, sync::mpsc, thread};

    use tempfile::tempdir;

    use super::*;

    #[test]
    fn test_env_vars() -> Result<(), Box<dyn Error>> {
        println!("running test...");
        let dir = tempdir()?;
        let path = dir.path().join("config.toml");

        let mut file = File::create_new(&path)?;
        writeln!(
            file,
            r#"
            [general]
            wallpaper_path = "$HOME/Pictures"
            interval = 60
            shuffle = true

            [transition]
            fps = 180
            "#
        )?;

        let config = Config::new(Some(path.to_str().unwrap())).unwrap_or_default();

        let (tx, rx) = mpsc::channel();
        let mut daemon = Daemon::new(config).unwrap();
        println!("running daemon..");
        let handle = thread::spawn(move || {
            daemon.run(rx);

            let conf_duration = normalize_duration(
                daemon.config.duration(),
                SCREENWH.0,
                SCREENWH.1,
                daemon.angle.unwrap(),
            );
            let env_duration = env::var("TRANSITION_DURATION").unwrap();
            assert_eq!(env_duration, conf_duration.to_string());
            let _ = tx.send(Commands::Next);

            let conf_duration = normalize_duration(
                daemon.config.duration(),
                SCREENWH.0,
                SCREENWH.1,
                daemon.angle.unwrap(),
            );
            let env_duration = env::var("TRANSITION_DURATION").unwrap();
            assert_eq!(env_duration, conf_duration.to_string());

            let _ = tx.send(Commands::Next);
            let conf_duration = normalize_duration(
                daemon.config.duration(),
                SCREENWH.0,
                SCREENWH.1,
                daemon.angle.unwrap(),
            );
            let env_duration = env::var("TRANSITION_DURATION").unwrap();
            assert_eq!(env_duration, conf_duration.to_string());

            let _ = tx.send(Commands::Shutdown);
        });

        handle.join().unwrap();

        Ok(())
    }
}
