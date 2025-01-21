use std::{env, error::Error, fs::File, io::Write, sync::mpsc};

use tempfile::tempdir;
use walrus::{commands::Commands, config::Config, daemon::Daemon, utils::normalize_duration};

#[test]
fn test_transition_vars() -> Result<(), Box<dyn Error>> {
    // need to make the transitions manually to assert that the environment variables are actually
    // set properly before the transition... it kinda seems like as if they're not changing quickly
    // enough sometimes...

    let dir = tempdir()?;
    let config_path = dir.path().join("config.toml");

    let mut file = File::create(&config_path)?;
    writeln!(
        file,
        r#"
        [general]
        wallpaper_path = "$HOME/Pictures"
        interval = 999
        shuffle = true
        
        [transition]
        fps = 180
        "#
    )?;
    let config = Config::new(Some(config_path.to_str().unwrap())).unwrap_or_default();

    let (tx, rx) = mpsc::channel();

    let mut daemon = Daemon::new(config).expect("Fatal: failed to initialize Walrus Daemon");
    daemon.run(rx);

    let env_duration =
        env::var("SWWW_TRANSITION_DURATION").expect("Failed to read duration environment variable");
    let duration_from_config = normalize_duration(daemon.config.duration(), 2560, 1440, 360.0);
    assert_eq!(env_duration, duration_from_config.to_string());
    let _ = tx.send(Commands::Next);

    let env_duration =
        env::var("SWWW_TRANSITION_DURATION").expect("Failed to read duration environment variable");
    let duration_from_config = normalize_duration(daemon.config.duration(), 2560, 1440, 360.0);
    assert_eq!(env_duration, duration_from_config.to_string());
    let _ = tx.send(Commands::Next);

    let _ = tx.send(Commands::Shutdown);
    Ok(())
}
