use std::fmt;
use std::fmt::Display;
use std::fs;
use std::os::unix;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::sync::mpsc;
use std::sync::mpsc::Receiver;
use std::time::Duration;

use rand::Rng;
use rand::SeedableRng;
use rand::rngs::SmallRng;
use rand::seq::SliceRandom;
use walkdir::WalkDir;
use walrus_core::commands::Commands;
use walrus_core::config::Config;
use walrus_core::config::Pos;
use walrus_core::config::Resolution;
use walrus_core::config::TransitionFlavour;
use walrus_core::config::WaveSize;
use walrus_core::ipc;

use crate::transition::TransitionArgBuilder;

#[derive(Debug)]
pub struct Daemon {
    pub config: Config,
    pub paused: bool,
    pub queue: Queue,
    rng: SmallRng,
}

impl Daemon {
    pub fn new(config: Config) -> Self {
        let directory = config.wallpaper_path();

        tracing::debug!("Starting with Config: {}", config);
        Self {
            config,
            paused: false,
            queue: Queue::new(&directory),
            rng: SmallRng::from_os_rng(),
        }
    }

    pub fn run(&mut self, rx: &Receiver<Commands>) {
        // TODO: have different sorting options (enum and match)
        if self.config.shuffle() {
            self.queue.shuffle();
        } else {
            self.queue.sort();
        }
        tracing::debug!("{:#?}", self.queue);

        // Set wallpaper initially.
        if let Some(wallpaper) = self.queue.get_current() {
            tracing::info!("Setting wallpaper: {}", wallpaper.display());
            self.set_wallpaper(wallpaper.clone().as_path());
        }

        let mut cont = true;
        while cont {
            let interval = self.config.interval();
            let timeout = Duration::from_secs(interval);

            match rx.recv_timeout(timeout) {
                Ok(Commands::Config) => unreachable!(),
                Ok(Commands::Categorise { category }) => {
                    self.create_category_symlink(self.queue.get_current().unwrap(), &category);
                }
                Ok(Commands::Next) => {
                    tracing::debug!("Received Next command");
                    self.next_wallpaper();
                }
                Ok(Commands::Pause) => {
                    tracing::debug!("Received Pause command");
                    self.pause();
                }
                Ok(Commands::Previous) => {
                    tracing::debug!("Received Previous command");
                    self.previous_wallpaper();
                }
                Ok(Commands::Resume) => {
                    tracing::debug!("Received Resume command");
                    self.resume();
                }
                /*
                 * Reload command is automatically called from Config::watch(). It is called on
                 * every modification event, including file removal. In the case of file removal
                 * the watcher checks whether a new file can be found.
                 */
                Ok(Commands::Reload) => {
                    tracing::debug!("Received Reload command");
                    self.reload_config();
                }
                Ok(Commands::Shutdown) => {
                    tracing::debug!("Received Shutdown command");
                    cont = false;
                }
                Err(mpsc::RecvTimeoutError::Timeout) => {
                    if !self.paused {
                        tracing::debug!("Timeout: changing wallpapers...");
                        self.next_wallpaper();
                        continue;
                    }
                    tracing::debug!("Timeout: paused, not changing wallpapers");
                }
                /*
                 * Unsure when this can happen. One such case is if there is an instance of walrus
                 * already running and another one is started.
                 * Since file locking was later implemented, that should not happen.
                 */
                Err(mpsc::RecvTimeoutError::Disconnected) => {
                    tracing::error!("Timeout: channel disconnected");
                    cont = false;
                }
            }
        }
    }

    fn create_category_symlink(&self, src: &Path, category: &str) {
        let base_path = self.config.wallpaper_path();
        let dir = base_path.join(format!(".{category}"));

        fs::create_dir_all(&dir).expect("Failed to create directories for: {dir:?}");

        let rel = src
            .strip_prefix(&base_path)
            .expect("Wallpaper prefix does not match base path");
        let dst = dir.join(rel);

        if let Some(parent) = dst.parent()
            && let Err(e) = fs::create_dir_all(parent)
        {
            tracing::error!("Error (mkdir {parent:?}): {e}");
            return;
        }

        tracing::debug!("Symlinking: {} <- {}", src.display(), dst.display());
        if let Err(e) = unix::fs::symlink(src, &dst) {
            tracing::error!(
                "Error creating symlink ({} <- {}): {e}",
                src.display(),
                dst.display()
            );
        }
    }

    fn new_transition(&mut self) -> Vec<String> {
        let resolution = self.config.resolution();

        let bezier = self.config.bezier();
        let duration = self.config.duration();
        let dynamic_duration = self.config.dynamic_duration();
        let fill = self.config.fill();
        let filter = self.config.filter();
        let fps = self.config.fps();
        let resize = self.config.resize();
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
            .with_fill(fill)
            .with_filter(filter)
            .with_fps(fps)
            .with_resize(resize)
            .with_step(step)
            .with_bezier(bezier);

        // This is a potentially potentially confusing reassignment?
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

    fn advance_wallpaper(&mut self, advance_fn: fn(&mut Queue)) {
        advance_fn(&mut self.queue);

        if let Some(current) = self.queue.get_current()
            && !current.exists()
        {
            tracing::warn!("Wallpaper in this position is missing, removing it from queue.");
            self.queue.cleanup_invalid_files();
        }

        if let Some(wallpaper) = self.queue.get_current() {
            let wallpaper = wallpaper.clone();
            tracing::info!("Setting wallpaper: {}", wallpaper.display());
            self.set_wallpaper(wallpaper.as_path());
        } else {
            tracing::error!("No valid path found in queue, shutting down");
            let _ = ipc::send_command(Commands::Shutdown);
        }
    }

    fn next_wallpaper(&mut self) {
        self.advance_wallpaper(Queue::next);
    }

    fn previous_wallpaper(&mut self) {
        self.advance_wallpaper(Queue::previous);
    }

    // TODO: Play/Pause could also be a toggle instead and just flip self.paused.
    fn pause(&mut self) {
        self.paused = true;
    }

    fn resume(&mut self) {
        self.paused = false;
    }

    fn set_wallpaper(&mut self, path: &Path) {
        let args = self.new_transition();

        let _ = Command::new(self.config.swww_path())
            .args(args)
            .arg(path)
            .spawn()
            .expect("Error spawning sww process")
            .wait();
    }

    // WARN:
    // Printing debug information from this function can be confusing because it might be called
    // multiple times. The reason is because the watcher polls and calls this function every time
    // it does that. For example Neovim uses atomic file writing, but other editors might do this
    // differently so the debug information depends on how the file is edited.
    fn reload_config(&mut self) {
        tracing::info!("Reloading config...");
        let config = Config::new().unwrap_or_else(|e| {
            tracing::error!("Error in config: {e}");
            tracing::warn!("Falling back to default config...");
            Config::default()
        });
        self.config = config;
    }
}

impl Display for Daemon {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}", self.queue)?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct Queue {
    queue: Vec<PathBuf>,
    index: usize,
}

impl Queue {
    fn new(dir: &Path) -> Self {
        Self {
            queue: WalkDir::new(dir)
                .follow_links(true)
                .into_iter()
                .filter_map(Result::ok)
                .filter(|entry| {
                    !entry
                        .path()
                        .components()
                        .nth(dir.components().count())
                        .and_then(|c| c.as_os_str().to_str())
                        .is_some_and(|s| s == ".like" || s == ".dislike")
                })
                .filter(|entry| entry.path().is_file())
                .map(|entry| entry.path().to_owned())
                .collect::<Vec<_>>(),
            index: 0,
        }
    }

    fn shuffle(&mut self) {
        let mut rng = rand::rng();
        // Might be confusing that he method is called shuffle so this kinda looks like a recursive call.
        self.queue.shuffle(&mut rng);
        self.index = 0;
    }

    fn sort(&mut self) {
        self.queue.sort();
    }

    fn get_current(&self) -> Option<&PathBuf> {
        self.queue.get(self.index)
    }

    fn next(&mut self) {
        if !self.queue.is_empty() {
            self.index = (self.index + 1) % self.queue.len();
        }
    }

    fn previous(&mut self) {
        if !self.queue.is_empty() {
            self.index = (self.index + self.queue.len() - 1) % self.queue.len();
        }
    }

    fn cleanup_invalid_files(&mut self) -> bool {
        let inital_len = self.queue.len();
        self.queue.retain(|path| path.exists());

        if !self.queue.is_empty() {
            self.index = self.index.min(self.queue.len() - 1);
        }

        self.queue.len() < inital_len
    }

    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }
}

impl Display for Queue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, wallpaper) in self.queue.iter().enumerate() {
            writeln!(f, "{i} - {}", wallpaper.display())?;
        }
        Ok(())
    }
}

pub fn normalize_duration(base_duration: f64, res: Resolution, angle_degrees: f32) -> f64 {
    let width = f64::from(res.width);
    let height = f64::from(res.height);

    let theta: f64 = angle_degrees.to_radians().into();
    let distance_at_angle = (width * theta.cos().abs()) + (height * theta.sin().abs());
    tracing::debug!("DistanceAtAngle: {distance_at_angle}");
    let diagonal_distance = (width.powi(2) + height.powi(2)).sqrt();
    let ratio = diagonal_distance / distance_at_angle;
    base_duration * ratio
}
