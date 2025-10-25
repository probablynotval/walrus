use bincode::{Decode, Encode, config};
use clap::{Parser, Subcommand};

#[derive(Clone, Parser)]
#[command(name = "Walrus", version = "0.1.2", about = "Convenient wrapper for swww with sensible defaults", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Clone, Debug, Decode, Encode, Subcommand)]
pub enum Commands {
    #[command(about = "Categorise current wallpaper")]
    Categorise { category: String },
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

impl Commands {
    pub fn to_bytes(&self) -> Option<Vec<u8>> {
        bincode::encode_to_vec(self, config::standard()).ok()
    }

    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        // This should never panic as the client and server follow the same protocol.
        let (decoded, _): (Self, _) = bincode::decode_from_slice(bytes, config::standard())
            .expect("Error decoding command from bytes");

        match decoded {
            // Config command should never reach the daemon.
            Commands::Config => None,
            _ => Some(decoded),
        }
    }
}
