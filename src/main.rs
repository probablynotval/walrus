use clap::{Parser, Subcommand};
use std::{thread, time::Duration};
use walrus::{config::Config, paths::Paths, set_wallpaper};

#[derive(Parser)]
#[command(name = "Walrus", version = "0.1.0", about = "SWWW wrapper", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    #[command(about = "Starts the program")]
    Init,
}

fn main() {
    let mut p = Paths::new().expect("Failed to initialize Paths object");
    if p.paths.is_empty() {
        println!("Paths is empty, exiting...");
        return;
    }
    let index = p.index;

    let config = Config::from("config.toml").unwrap_or_default();
    let general = config.general.unwrap_or_default();
    let interval = general.interval.unwrap_or_default();
    let shuffle = general.shuffle.unwrap_or_default();

    let cli = Cli::parse();
    match &cli.command {
        Commands::Init => loop {
            if shuffle {
                p.shuffle();
            }
            for path in &p.paths {
                println!("Changing wallpaper: {path}");
                set_wallpaper(path.as_str());
                thread::sleep(Duration::from_secs(interval));
            }
        },
    }
}
