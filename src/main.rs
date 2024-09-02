use clap::{Parser, Subcommand};
use std::{env, path::PathBuf, thread, time::Duration};
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
    #[command(about = "Sets the path where Walrus will recursively look for images")]
    Directory {
        #[arg(
            env = "WALRUS_DIR",
            help = "Sets the path where Walrus will recursively look for images"
        )]
        path: String,

        #[arg(short, long, help = "Prints the current directory")]
        get: bool,
    },
}

fn main() {
    let config = Config::from("config.toml").unwrap_or_default();
    let general = config.general.unwrap_or_default();
    let interval = general.interval.unwrap_or_default();
    let shuffle = general.shuffle.unwrap_or_default();

    let cli = Cli::parse();
    match &cli.command {
        Commands::Init => {
            let mut p = Paths::new().expect("Failed to initialize Paths object");

            if p.paths.is_empty() {
                println!("Paths is empty, exiting...");
                return;
            }

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
        Commands::Directory { path, get } => {
            if *get {
                println!("Current Walrus directory is: {:#?}", env::var("WALRUS_DIR"))
            } else {
                env::set_var("WALRUS_DIR", PathBuf::from(path).display().to_string());
                println!("DEBUG: Walrus directory set: {:#?}", env::var("WALRUS_DIR"))
            }
        }
    }
}
