mod commands;

use commands::favorite::favorite;

use std::{path::PathBuf, str, process::Command};
use log::{info, debug, error, log_enabled, Level, LevelFilter};
use env_logger::{Builder, Env, Target};
use clap::{Parser, Subcommand};
use regex::Regex;

#[derive(Parser)]
#[command(name = "WallFlick")]
#[command(version = "0.1.0")]
#[command(about = "SWWW manager", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    #[command(about = "Adds or removes current wallpaper from favorites")]
    Favorite,
    #[command(about = "Skip ahead to the next wallpaper")]
    Next,
    #[command(about = "Go back to the previous wallpaper")]
    Previous,
    #[command(about = "Reshuffles the queue")]
    Shuffle,
    #[command(about = "How frequently to change wallpaper")]
    SwapInterval {
        #[arg(short, long, default_value_t = 300)]
        interval: u32,
    },
    #[command(about = "Removes simple and none transitions (and fade if specified)")]
    BetterRandom,
}


fn get_current() -> Option<PathBuf> {
    let output = Command::new("/usr/bin/swww")
        .arg("query")
        .output()
        .expect("[swww query] Failed to get current wallpaper");
    debug!("{output:#?}");

    let output_str = str::from_utf8(&output.stdout)
        .expect("[swww query] Failed to convert query to utf8");
    debug!("{output_str}");

    let pattern = Regex::new(r".*\.[\w:]+")
        .expect("[swww query] Failed to create regex");

    if let Some(captures) = pattern.captures(output_str) {
        debug!("{captures:#?}");
        if let Some(path) = captures.get(1) {
            debug!("{path:#?}");
            return Some(PathBuf::from(path.as_str()));
        }
    }

    None
}

fn main() {
    let mut logger = Builder::new();
    logger
        .filter(None, LevelFilter::Debug)
        .target(Target::Stderr)
        .init();

    let cli = Cli::parse();

    match &cli.command {
        Commands::Favorite => favorite(),
        _ => println!("Not implemented"),
    }
}
