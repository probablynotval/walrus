use crate::config::*;

use walkdir::WalkDir;

pub struct Paths {
    pub paths: Vec<String>,
}

impl Paths {
    pub fn new() -> Option<Self> {
        // Get directory path from config
        let directory = Config::from("config.toml")
            .unwrap()
            .general
            .unwrap()
            .path
            .unwrap();

        let mut paths = Vec::new();
        for entry in WalkDir::new(directory).follow_links(true) {
            let entry = entry.unwrap();
            if entry.path().is_file() {
                if let Some(path) = entry.path().to_str() {
                    paths.push(path.to_string());
                }
            }
        }
        Some(Self { paths })
    }
}
