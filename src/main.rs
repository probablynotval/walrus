use clap::{Parser, Subcommand};
use std::{env, path::PathBuf};

const DEFAULT_PATH: &str = "$HOME/Pictures/Wallpapers";

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
        #[arg(env = "WALRUS_DIR", default_value_t = String::from(DEFAULT_PATH), help= "Sets the path where Walrus will recursively look for images")]
        path: String,

        #[arg(short, long, help = "Prints the current directory")]
        get: bool,
    },
}

fn main() {
    let cli = Cli::parse();
    match &cli.command {
        Commands::Init => {
            todo!()
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
