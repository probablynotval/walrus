use crate::{config::*, set_wallpaper};

use rand::{seq::SliceRandom, thread_rng};
use walkdir::WalkDir;

pub struct Paths {
    pub paths: Vec<String>,
    pub index: usize,
}

impl Paths {
    pub fn new() -> Option<Self> {
        // Get directory path from config
        let config = Config::from("config.toml").unwrap();
        let directory = config.general.unwrap().path.unwrap();
        let mut paths = Vec::new();
        for entry in WalkDir::new(directory).follow_links(true) {
            let entry = entry.unwrap();
            if entry.path().is_file() {
                if let Some(path) = entry.path().to_str() {
                    paths.push(path.to_string());
                }
            }
        }
        Some(Self { paths, index: 0 })
    }

    fn shuffle(&mut self) {
        let mut rng = thread_rng();
        self.paths.shuffle(&mut rng);
        self.index = 0;
        let _ = set_wallpaper();
    }
}
