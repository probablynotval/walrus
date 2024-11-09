pub mod commands;
pub mod config;
pub mod handler;
pub mod ipc;

use config::Config;
use env_logger::Target;
use log::{debug, trace, LevelFilter};
use rand::Rng;
use std::{env, error::Error, process::Command};

// TODO: Make these functions part of Walrus object
pub fn set_wallpaper(path: &str, config: Config) {
    get_transition(config.clone());

    let swww_path = config
        .clone()
        .general
        .unwrap_or_default()
        .swww_path
        .unwrap_or_default();

    let _ = Command::new(swww_path)
        .arg("img")
        .arg(path)
        .spawn()
        .unwrap();
}

fn get_transition(config: Config) {
    // Working on implementing values from config here
    let transition = config.transition.unwrap_or_default();
    let bezier = transition.bezier.unwrap_or_default();
    let duration = transition.duration.unwrap_or_default();
    let dynamic_duration = transition.dynamic_duration.unwrap_or_default();
    let flavour = transition.flavour.unwrap_or_default();
    let fps = transition.fps.unwrap_or_default();
    let step = transition.step.unwrap_or_default();
    let (wave_width_min, wave_width_max, wave_height_min, wave_height_max) =
        transition.wave_size.unwrap_or_default();

    env::set_var(
        "SWWW_TRANSITION_BEZIER",
        format!("{},{},{},{}", bezier[0], bezier[1], bezier[2], bezier[3]),
    );
    env::set_var("SWWW_TRANSITION_DURATION", duration.to_string());
    env::set_var("SWWW_TRANSITION_FPS", fps.to_string());
    env::set_var("SWWW_TRANSITION_STEP", step.to_string());

    let mut rng = rand::thread_rng();
    let flavour_rng = rng.gen_range(0..flavour.len());
    let flavour_selection = flavour.get(flavour_rng).unwrap().to_string();
    debug!("Flavour: {flavour_selection}");

    // TODO: Make flavours into concrete types
    match flavour_selection.as_str() {
        "wipe" | "wave" => {
            let angle_rng = rng.gen_range(0.0..360.0);
            trace!("Angle: {angle_rng}");
            if dynamic_duration {
                let normalized_duration = normalize_duration(duration, 2560.0, 1440.0, angle_rng);
                env::set_var("SWWW_TRANSITION_DURATION", normalized_duration.to_string());
                trace!("Dynamic duration: {normalized_duration}");
            }
            trace!("Duration: {duration}");
            env::set_var("SWWW_TRANSITION_ANGLE", angle_rng.to_string());

            if flavour_selection.as_str() == "wipe" {
                env::set_var("SWWW_TRANSITION", "wipe");
            } else {
                let width_wave_rng = rng.gen_range(wave_width_min..=wave_width_max);
                let height_wave_rng = rng.gen_range(wave_height_min..=wave_height_max);
                env::set_var(
                    "SWWW_TRANSITION_WAVE",
                    format!("{},{}", width_wave_rng, height_wave_rng),
                );
                env::set_var("SWWW_TRANSITION", "wave");
            }
        }
        "grow" | "outer" => {
            if flavour_selection.as_str() == "grow" {
                env::set_var("SWWW_TRANSITION", "grow");
            } else {
                env::set_var("SWWW_TRANSITION", "outer");
            };

            let x_position_rng = rng.gen::<f64>();
            let y_position_rng = rng.gen::<f64>();
            env::set_var(
                "SWWW_TRANSITION_POS",
                format!("{},{}", x_position_rng, y_position_rng),
            );
        }
        &_ => unimplemented!(),
    };
}

pub fn normalize_duration(base_duration: f64, width: f64, height: f64, angle_degrees: f64) -> f64 {
    let theta = angle_degrees.to_radians();
    let distance_at_angle = (width * theta.sin().abs()) + (height * theta.cos().abs());
    let diagonal_distance = (width.powi(2) + height.powi(2)).sqrt();
    let ratio = diagonal_distance / distance_at_angle;
    base_duration * ratio
}

pub fn init_logger(debug_level: LevelFilter) -> Result<(), Box<dyn Error>> {
    env_logger::builder()
        .filter(None, debug_level)
        .target(Target::Stderr)
        .format_indent(Some(4))
        .format_timestamp_millis()
        .try_init()?;
    Ok(())
}
