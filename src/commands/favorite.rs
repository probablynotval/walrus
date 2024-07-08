use std::{path::Path, os::linux::fs::MetadataExt, io::{self, Read}, fs::{create_dir_all, copy, remove_file, metadata, File}};
use crate::get_current;

use log::{info, debug, error};


fn compare_files(file1: &Path, file2: &Path) -> io::Result<bool> {
    // Return early if the files are not the same size
    if metadata(file1)?.st_size() != metadata(file2)?.st_size() {
        debug!("{file1:#?} and {file2:#?} are not of equal size");
        return Ok(false);
    }

    let mut f1 = File::open(file1)?;
    let mut f2 = File::open(file2)?;

    let mut buffer1 = [0; 1024];
    let mut buffer2 = [0; 1024];

    loop {
        let n1 = f1.read(&mut buffer1)?;
        let n2 = f2.read(&mut buffer2)?;

        if n1 != n2 {
            return Ok(false);
        }

        if n1 == 0 {
            break;
        }

        if buffer1[..n1] != buffer2[..n1] {
            return Ok(false);
        }
    }

    Ok(true)
}

// TODO: Convert the image into a webp before adding it to faves to save space
pub fn favorite() {
    // FIXME: should be replaced for environment variable
    let current = match get_current() {
        Ok(path) => path,
        Err(e) => panic!("error :'( ---> {e}"),
    };

    // Current wallpaper's file name
    let current_file = match current.file_name() {
        Some(name) => name,
        None => {
            error!("Current wallpaper has no file name");
            return;
        },
    };

    // Path to the file's directory
    let dir = match current.parent() {
        Some(parent) => parent,
        None => {
            error!("Current wallpaper has no parent directory");
            return;
        },
    };

    // Path to .../wallpapers/favorite
    let favorites_dir = dir.join("favorites");
    debug!("------> favorites dir {favorites_dir:#?}");

    // Create directories if they don't exist
    if let Err(e) = create_dir_all(&favorites_dir) {
        error!("Failed to create directories: {e:#?}");
        return;
    }

    let favorite_file = favorites_dir.join(current_file);

    // Check if file is already favorite
    if favorite_file.exists() {
        debug!("Checking if {favorite_file:#?} is already in favorites...");
        match compare_files(&current, &favorite_file) {
            Ok(true) => {
                info!("Removing wallpaper from favorites: {favorite_file:#?}");
                if let Err(e) = remove_file(&favorite_file) {
                    error!("Failed to remove wallpaper from favorites: {e:#?}");
                }
                return;
            }
            // TODO: add better implementation to handle conflicts
            Ok(false) => {
                info!("Conflict error: a different file with the same name as {favorite_file:?} \
                exists in the directory");
                return;
            }
            Err(e) => {
                error!("Failed to compare files: {e:#?}");
                return;
            }
        }
    }

    // Copy file to .../wallpapers/favorite
    if let Err(e) = copy(&current, &favorite_file) {
        debug!("{current:#?}");
        debug!("{favorite_file:#?}");
        error!("Failed to copy file to favorites: {e:#?}");
    } else {
        info!("Added {favorite_file:#?} to favorites");
    }
}
