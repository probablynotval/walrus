use walrus::config::*;

use std::error::Error;
use std::fs::File;
use std::io::Write;
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
        
        [transition]
        interval = 60
        fps = 180
    "#
    )?;

    let path_str = config_path.to_str().unwrap();

    // Test the Config::from function
    let config = Config::from(path_str)?;

    assert_eq!(
        *config.general.as_ref().unwrap().path.as_ref().unwrap(),
        String::from("$HOME/Pictures")
    );
    assert_eq!(
        *config
            .transition
            .as_ref()
            .unwrap()
            .interval
            .as_ref()
            .unwrap(),
        60 as usize
    );
    assert_eq!(
        *config.transition.as_ref().unwrap().fps.as_ref().unwrap(),
        180 as usize
    );

    Ok(())
}
