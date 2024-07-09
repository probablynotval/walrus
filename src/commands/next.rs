// TODO: Add next function
// FIXME: ??? if images are added then what ???

use log::{debug, error, info};
use rand::{seq::SliceRandom, thread_rng};
use std::{fs, process::Command};
use walkdir::WalkDir;

use crate::get_current;

// TODO: Probably move this function to main.rs since it's a fundamental part of the program
pub fn shuffle(interval: &Option<u32>) -> Result<(), std::io::Error> {
    // Get wallpaper directory
    let file_path = get_current().unwrap();
    // Remove file from directory path
    let current = file_path.parent().unwrap();

    // Create vector to hold paths
    let mut paths = Vec::new();

    // Recursively search directory for wallpapers
    for entry in WalkDir::new(current).follow_links(true) {
        // Assert that entry is DirEntry and not error
        let entry = entry?;
        // If the path is valid push it to the vector (what would be an invalid path? is it possible?)
        if entry.path().is_file() {
            if let Some(path) = entry.path().to_str() {
                paths.push(path.to_string());
            }
        }
    }

    let mut rng = thread_rng();
    paths.shuffle(&mut rng);
    debug!("{paths:#?}");

    Ok(())
}

pub fn next() {
    println!("WIP");
}
