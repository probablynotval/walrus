use clap::{Parser, Subcommand};
use num_enum::{IntoPrimitive, TryFromPrimitive};

#[derive(Clone, Parser)]
#[command(name = "Walrus", version = "0.1.2", about = "Convenient wrapper for swww with sensible defaults", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Clone, Copy, Debug, IntoPrimitive, TryFromPrimitive, Subcommand)]
#[repr(u8)]
pub enum Commands {
    #[command(about = "Prints config")]
    Config,
    #[command(about = "Dislike")]
    Dislike,
    #[command(about = "Like")]
    Like,
    #[command(about = "Go to the next wallpaper in queue")]
    Next,
    #[command(about = "Pause the playback")]
    Pause,
    #[command(about = "Go to the previous wallpaper in queue")]
    Previous,
    #[command(about = "Resume the playback")]
    Resume,
    #[command(about = "Stops the program")]
    Shutdown,
    #[command(hide = true)]
    Reload,
}
