use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "Walrus", version = "0.1.0", about = "SWWW wrapper", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
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
}
