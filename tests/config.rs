use walrus::config::*;

use directories::UserDirs;
use std::{error::Error, fs::File, io::Write, path::PathBuf};
use tempfile::tempdir;

#[test]
fn test_config_from() -> Result<(), Box<dyn Error>> {
    let dir = tempdir()?;
    let config_path = dir.path().join("config.toml");

    // Create a sample config.toml file
    let mut file = File::create(&config_path)?;
    writeln!(
        file,
        r#"
        [general]
        path = "$HOME/Pictures"
        interval = 60
        shuffle = true
        
        [transition]
        fps = 180
    "#
    )?;

    let path_str = config_path.to_str().unwrap();

    // Test the Config::from function
    let config = Config::from(path_str)?;

    assert_eq!(
        *config.general.as_ref().unwrap().path.as_ref().unwrap(),
        PathBuf::from("$HOME/Pictures")
    );
    assert_eq!(
        *config.general.as_ref().unwrap().interval.as_ref().unwrap(),
        60 as u64
    );
    assert_eq!(
        *config.transition.as_ref().unwrap().fps.as_ref().unwrap(),
        180 as u32
    );

    Ok(())
}

#[test]
fn test_config_from_defaults() -> Result<(), Box<dyn Error>> {
    let config = Config::default();
    let general = config.clone().general.unwrap_or_default();
    let path = general.path.unwrap_or_default();
    let interval = general.interval.unwrap_or_default();
    let transition = config.clone().transition.unwrap_or_default();
    let fps = transition.fps.unwrap_or_default();

    assert_eq!(
        path,
        UserDirs::new()
            .unwrap()
            .picture_dir()
            .unwrap()
            .join("Wallpapers")
    );
    assert_eq!(interval, 300 as u64);
    assert_eq!(fps, 60 as u32);

    Ok(())
}
