use clap::{Parser, Subcommand};

#[derive(Clone, Parser)]
#[command(name = "Walrus", version = "0.1.2", about = "Convenient wrapper for SWWW with sensible defaults", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Clone, Copy, Debug, Subcommand)]
pub enum Commands {
    #[command(about = "Prints config")]
    Config,
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
