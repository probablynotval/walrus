use std::{
    fmt::{self, Display},
    path::PathBuf,
    process::Command,
    sync::mpsc::{self, Receiver},
    time::Duration,
};

use log::{debug, error, info, warn};
use rand::{Rng, SeedableRng, rngs::SmallRng, seq::SliceRandom};
use walkdir::WalkDir;

use crate::{
    commands::Commands,
    config::{Config, TransitionFlavour},
    ipc::send_ipc_command,
    transition::{Pos, TransitionArgBuilder, WaveSize},
    utils::{decrement_index, increment_index, normalize_duration},
};

pub struct Daemon {
    pub config: Config,
    pub index: usize,
    pub paused: bool,
    pub queue: Vec<String>,
    // TODO: this stuff would probably move to the TransitionBuilder?
    rng: SmallRng,
}

impl Daemon {
    pub fn new(config: Config) -> Option<Self> {
        let directory = config.wallpaper_path();
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
            config,
            paused: false,
            queue,
            index: 0,
            rng: SmallRng::from_os_rng(),
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
                /*
                 * Reload command is automatically called from Config::watch(). It is called on
                 * every modification event, including file removal. In the case of file removal
                 * the watcher checks whether a new file can be found.
                 */
                Ok(Commands::Reload) => {
                    debug!("Received Reload command");
                    self.reload_config();
                }
                Ok(Commands::Shutdown) => {
                    debug!("Received Shutdown command");
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
                /*
                 * Unsure when this can happen. One such case is if there is an instance of walrus
                 * already running and another one is started.
                 */
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

    fn new_transition(&mut self) -> Vec<String> {
        let resolution = self.config.resolution();

        let bezier = self.config.bezier();
        let duration = self.config.duration();
        let dynamic_duration = self.config.dynamic_duration();
        let fps = self.config.fps();
        let step = self.config.step();

        let flavours = self.config.flavour();
        let flavour_rng = self.rng.random_range(0..flavours.len());
        let flavour = flavours.get(flavour_rng).unwrap();

        let angle = self.rng.random_range(0.0..360.0);

        let duration = match flavour {
            TransitionFlavour::Wipe | TransitionFlavour::Wave if dynamic_duration => {
                normalize_duration(duration, resolution, angle)
            }
            _ => duration,
        };

        let builder = TransitionArgBuilder::new()
            .with_transition(flavour)
            .with_duration(duration)
            .with_fps(fps)
            .with_step(step)
            .with_bezier(bezier);

        let builder = match flavour {
            TransitionFlavour::Wipe => builder.with_angle(angle),
            TransitionFlavour::Wave => {
                let (width_min, width_max, height_min, height_max) = self.config.wave_size();
                let width = self.rng.random_range(width_min..=width_max);
                let height = self.rng.random_range(height_min..=height_max);
                let wave = WaveSize { width, height };

                let angle = (360.0 + angle - 90.0) % 360.0;

                builder.with_angle(angle).with_wave(wave)
            }
            TransitionFlavour::Grow | TransitionFlavour::Outer => {
                let x: f32 = self.rng.random_range(0.0..=1.0);
                let y: f32 = self.rng.random_range(0.0..=1.0);
                builder.with_pos(Pos { x, y })
            }
        };

        builder.build()
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
        let mut rng = rand::rng();
        self.queue.shuffle(&mut rng);
        self.index = 0;
    }

    fn set_wallpaper(&mut self, path: String) {
        let args = self.new_transition();

        let _ = Command::new(self.config.swww_path())
            .args(args)
            .arg(path)
            .spawn()
            .expect("Error spawning process")
            .wait();
    }

    // NOTE:
    // Printing debug information from this function can be confusing because it might be called
    // multiple times. The reason is because the watcher polls and calls this function every time
    // it does that. For example Neovim uses atomic file writing, but other editors might do this
    // differently so the debug information depends on how the file is edited.
    fn reload_config(&mut self) {
        info!("Reloading config...");
        let config = Config::new().unwrap_or_default();
        self.config = config;
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
