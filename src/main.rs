mod commands;

use commands::*;
use favorite::favorite;
use wallflick::{get_transition, set_wallpaper, Paths};

use clap::Parser;
use env_logger::{Builder, Target};
use log::LevelFilter;
use std::{boxed::Box, env, error::Error, result::Result, thread::sleep, time::Duration};

fn main() -> Result<(), Box<dyn Error + 'static>> {
    env::set_var("RUST_BACKTRACE", "1");
    let mut logger = Builder::new();
    logger
        .filter(None, LevelFilter::Debug)
        .target(Target::Stderr)
        .init();

    // Animation
    // Should be equal to refresh rate, no benefit setting it lower
    env::set_var("SWWW_TRANSITION_FPS", "180");
    // Lower for smoother animation
    env::set_var("SWWW_TRANSITION_STEP", "160");
    env::set_var("SWWW_TRANSITION_DURATION", "0.75");

    let mut paths = Paths::new()?;

    let cli = Cli::parse();
    match &cli.command {
        Commands::Init { interval } => {
            paths.shuffle();
            loop {
                get_transition();
                set_wallpaper(&paths)?;
                sleep(Duration::from_secs(*interval));
                paths.next_wallpaper();
            }
        }
        Commands::Favorite => favorite(),
        Commands::Shuffle => paths.shuffle(),
        Commands::Next => paths.next_wallpaper(),
        Commands::Previous => paths.prev_wallpaper(),
        Commands::Playback => {
            todo!()
        }
        Commands::BetterTransitions => {
            todo!();
        }
        Commands::Env {
            wave_size,
            bezier,
            fps,
            step,
            duration,
        } => {
            if !wave_size.is_empty() {
                todo!();
            };

            if !bezier.is_empty() {
                todo!();
            };

            if !fps.is_empty() {
                todo!();
            };

            if !step.is_empty() {
                todo!();
            };

            if !duration.is_empty() {
                todo!();
            };
        }
    }

    Ok(())
}
