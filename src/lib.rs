pub mod commands;
pub mod config;
pub mod paths;

use config::Config;
use rand::{thread_rng, Rng};
use std::{env, process::Command};

pub fn set_wallpaper(path: &str) {
    get_transition();

    let _ = Command::new("/usr/bin/swww")
        .arg("img")
        .arg(path)
        .spawn()
        .unwrap();
}

fn get_transition() {
    let flavour = ["wipe", "wave", "grow", "outer"];

    let flavour_rng = thread_rng().gen_range(0..flavour.len());
    let flavour_selection = flavour.get(flavour_rng).unwrap().to_string();

    match flavour_selection.as_str() {
        "wipe" | "wave" => {
            let angle_rng = thread_rng().gen_range(0..360);
            env::set_var("SWWW_TRANSITION_ANGLE", angle_rng.to_string());

            if flavour_selection.as_str() == "wipe" {
                env::set_var("SWWW_TRANSITION_BEZIER", ".48,0,.52,1"); // TODO: replace placeholder
                env::set_var("SWWW_TRANSITION", "wipe");
            } else {
                let width_wave_rng = thread_rng().gen_range(15..25);
                let height_wave_rng = thread_rng().gen_range(15..25);
                env::set_var(
                    "SWWW_TRANSITION_WAVE",
                    format!("{},{}", width_wave_rng, height_wave_rng),
                );
                env::set_var("SWWW_TRANSITION_BEZIER", ".48,0,.52,1"); // TODO: replace placeholder
                env::set_var("SWWW_TRANSITION", "wave");
            }
        }
        "grow" | "outer" => {
            if flavour_selection.as_str() == "grow" {
                env::set_var("SWWW_TRANSITION_BEZIER", ".48,0,.52,1"); // TODO: replace placeholder
                env::set_var("SWWW_TRANSITION", "grow");
            } else {
                env::set_var("SWWW_TRANSITION_BEZIER", ".48,0,.52,1"); // TODO: replace placeholder
                env::set_var("SWWW_TRANSITION", "outer");
            };

            let x_position_rng = thread_rng().gen_range(0..=1);
            let y_position_rng = thread_rng().gen_range(0..=1);
            env::set_var(
                "SWWW_TRANSITION_POS",
                format!("{},{}", x_position_rng, y_position_rng),
            );
        }
        &_ => unimplemented!(),
    };

    // Working on implementing values from config here
    let config = Config::from("config.toml").unwrap_or_default();
    let transition = config.transition.unwrap_or_default();
    let duration = transition.duration.unwrap_or_default();
    let fps = transition.fps.unwrap_or_default();
    let step = transition.step.unwrap_or_default();

    env::set_var("SWWW_TRANSITION_DURATION", duration.to_string());
    env::set_var("SWWW_TRANSITION_FPS", fps.to_string());
    env::set_var("SWWW_TRANSITION_STEP", step.to_string());
}
