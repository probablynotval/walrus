use clap::Parser;
use std::{thread, time::Duration};
use walrus::{
    commands::{Cli, Commands},
    config::Config,
    paths::Paths,
    set_wallpaper,
};

fn main() {
    let mut p = Paths::new().expect("Failed to initialize Paths object");
    if p.paths.is_empty() {
        println!("Paths is empty, exiting...");
        return;
    }
    let index = p.index;

    let config = Config::from("config.toml").unwrap_or_default();

    let cli = Cli::parse();
    match &cli.command {
        Some(Commands::Config) => {
            println!("{config}");
        }
        None => {
            let general = config.general.unwrap_or_default();
            let interval = general.interval.unwrap_or_default();
            let shuffle = general.shuffle.unwrap_or_default();
            loop {
                if shuffle {
                    p.shuffle();
                }
                for path in &p.paths {
                    println!("Changing wallpaper: {path}");
                    set_wallpaper(path.as_str());
                    thread::sleep(Duration::from_secs(interval));
                }
            }
        }
    }
}
