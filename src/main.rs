mod commands;

use std::{path::PathBuf, str, process::Command};
use log::{info, debug, error, log_enabled, Level, LevelFilter};
use env_logger::{Builder, Env, Target};
use clap::{Parser, Subcommand};
use regex::Regex;


fn get_current() -> Option<PathBuf> {
    let mut output = Command::new("/usr/bin/swww")
        .arg("query")
        .output()
        .expect("[swww query] Failed to get current wallpaper");
    debug!("{output:#?}");

    let output_str = str::from_utf8(&output.stdout)
        .expect("[swww query] Failed to convert query to utf8");
    debug!("{output_str}");

    let pattern = Regex::new(r"currently displaying: image: .*\.[\w:]+")
        .expect("[swww query] Failed to create regex");

    if let Some(captures) = pattern.captures(output_str) {
        debug!("{captures:#?}");
        if let Some(path) = captures.get(0) {
            debug!("{path:#?}");
            return Some(PathBuf::from(path.as_str()));
        }
    }

    None
}

fn main() {
    let mut builder = Builder::new();
    builder
        .filter(None, LevelFilter::Debug)
        .target(Target::Stderr)
        .init();

    match get_current() {
        Some(path) => println!("Current wallpaper: {path:?}"),
        None => println!("Wallpaper is None"),
    }
}
