mod commands;

use commands::favorite::favorite;

use clap::{Parser, Subcommand};
use env_logger::{Builder, Target};
use log::{debug, LevelFilter};
use regex::Regex;
use std::{path::PathBuf, process::Command, str};

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

fn get_current() -> anyhow::Result<PathBuf> {
    let output = Command::new("/usr/bin/swww")
        .arg("query")
        .output()
        .expect("[swww query] Failed to get current wallpaper");

    let output_str =
        str::from_utf8(&output.stdout).expect("[swww query] Failed to convert query to utf8");

    let pattern = Regex::new(r"\/.*\.[\w:]+")
        .expect("[swww query] Failed to create regex");

    if let Some(matches) = pattern.find(output_str) {
        let path = matches.as_str();
        return Ok(PathBuf::from(path));
    }

    Err(anyhow::anyhow!("[swww query] No valid path found"))
}

fn main() {
    let mut logger = Builder::new();
    logger.filter(None, LevelFilter::Debug).target(Target::Stderr).init();

    let cli = Cli::parse();
    match &cli.command {
        Commands::Favorite => favorite(),
        _ => println!("To be implemented"),
    }
}
