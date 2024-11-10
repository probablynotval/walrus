use crate::{commands::Commands, config::Config, set_wallpaper};
use log::{debug, error, info, trace};
use rand::{seq::SliceRandom, thread_rng};
use std::{
    fmt, fs,
    path::Path,
    sync::mpsc::{self, Receiver},
    time::Duration,
};
use walkdir::WalkDir;

#[derive(Clone)]
pub struct Walrus {
    pub config: Config,
    pub queue: Vec<String>,
    pub index: usize,
}

impl Walrus {
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
                Ok(Commands::Previous) => {
                    debug!("Received Previous command");
                    self.previous_wallpaper();
                }
                Ok(Commands::Shutdown) => {
                    debug!("Received Shutdown command");
                    if Path::new("/tmp/walrus.sock").exists() {
                        let _ = fs::remove_file("/tmp/walrus.sock");
                    }
                    cont = false;
                }
                Err(mpsc::RecvTimeoutError::Timeout) => {
                    debug!("Timeout: changing wallpapers...");
                    self.next_wallpaper();
                }
                Err(mpsc::RecvTimeoutError::Disconnected) => {
                    error!("Timeout: channel disconnected");
                    cont = false;
                }
            }
        }
    }

    fn current_wallpaper(&self) {
        if let Some(wallpaper) = self.queue.get(self.index) {
            info!("Setting wallpaper: {wallpaper}");
            set_wallpaper(wallpaper.as_str(), self.config.clone());
        }
    }

    fn next_wallpaper(&mut self) {
        self.index = (self.index + 1) % self.queue.len();
        if let Some(wallpaper) = self.queue.get(self.index) {
            info!("Setting wallpaper: {wallpaper}");
            set_wallpaper(wallpaper.as_str(), self.config.clone());
        }
    }

    fn previous_wallpaper(&mut self) {
        self.index = (self.index + self.queue.len() - 1) % self.queue.len();
        if let Some(wallpaper) = self.queue.get(self.index) {
            info!("Setting wallpaper: {wallpaper}");
            set_wallpaper(wallpaper.as_str(), self.config.clone());
        }
    }

    fn shuffle_queue(&mut self) {
        let mut rng = thread_rng();
        self.queue.shuffle(&mut rng);
        self.index = 0;
    }
}

impl fmt::Display for Walrus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, wallpaper) in self.queue.iter().enumerate() {
            writeln!(f, "{} - {}", i, wallpaper)?;
        }
        Ok(())
    }
}
