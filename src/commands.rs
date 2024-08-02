pub mod favorite;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "WallFlick")]
#[command(version = "0.1.0")]
#[command(about = "SWWW manager", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    #[command(about = "Starts the program")]
    Init {
        #[arg(
            short,
            long,
            default_value_t = 300,
            help = "Adjusts how often the wallpaper is changed"
        )]
        interval: u64,
    },
    #[command(about = "Adds or removes current wallpaper from favorites")]
    Favorite, // works but might need rewrite to fit in
    #[command(about = "Skip ahead to the next wallpaper")]
    Next,
    #[command(about = "Go back to the previous wallpaper")]
    Previous,
    #[command(about = "Reshuffles the queue")]
    Shuffle,
    #[command(about = "Play/pause the switching of wallpapers")]
    Playback, // not a clue on how to implement this atm
    #[command(about = "Removes simple and none transitions (and fade if specified)")]
    BetterTransitions, // can be something simple like bool func param for the transition function
    #[command(about = "Adjusts some animation variables")]
    Env {
        #[arg(
            short,
            long,
            default_value_t = String::from("10,20,10,20"),
            help = "Adjusts the size of the min and max size of the wave [min_x,min_y,max_x,max_y]"
        )]
        wave_size: String,
        #[arg(short, long, default_value_t = String::from(".48,0.52,1"), help = "Adjusts the transition bezier")]
        bezier: String,
        #[arg(
            short,
            long,
            default_value_t = String::from("180"),
            help = "Adjusts the framerate of the animation, set it to the refresh rate of your monitor"
        )]
        fps: String,
        #[arg(
            short,
            long,
            default_value_t = String::from("160"),
            help = "Adjusts the transition step, see 'swww img --help'"
        )]
        step: String,
        #[arg(short, long, default_value_t = String::from("0.75"), help = "Adjusts the duration of the transition")]
        duration: String,
    },
}
