pub mod config;
pub mod paths;

use paths::Paths;
use rand::{thread_rng, Rng};
use std::{env, process::Command};

pub fn set_wallpaper() {
    get_transition();

    let paths = Paths::new()
        .unwrap_or_else(|| todo!("Handle error case for initiating Paths object"))
        .paths;
    let path = paths.get(0).unwrap().as_str();
    // NOTE: Need to finish paths object to finish this
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
}
