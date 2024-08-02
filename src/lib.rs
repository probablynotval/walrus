use rand::{seq::SliceRandom, thread_rng, Rng};
use regex::Regex;
use std::{
    env,
    error::Error,
    ops::{Deref, DerefMut},
    path::{Path, PathBuf},
    process::Command,
    str,
    string::String,
};
use walkdir::WalkDir;

pub struct Paths {
    paths: Vec<String>,
    index: usize,
}

impl Paths {
    pub fn new() -> anyhow::Result<Self> {
        let current = get_current()?;
        let paths = Self::get_paths(&current)?;

        Ok(Self { paths, index: 0 })
    }

    // FIXME: this implementation has a fatal flaw, since it only looks at the dir of the current
    // wallpaper it might get stuck only looping through that directory
    fn get_paths(current: &Path) -> std::result::Result<Vec<String>, std::io::Error> {
        let mut paths = Vec::new();
        for entry in WalkDir::new(current.parent().unwrap()).follow_links(true) {
            let entry = entry?;
            if entry.path().is_file() {
                if let Some(path) = entry.path().to_str() {
                    paths.push(path.to_string());
                }
            }
        }
        Ok(paths)
    }

    pub fn shuffle(&mut self) {
        let mut rng = thread_rng();
        self.paths.shuffle(&mut rng);
        self.index = 0;
    }

    pub fn get_all_paths(&self) -> &Vec<String> {
        &self.paths
    }

    pub fn current(&self) -> Option<&String> {
        self.paths.get(self.index)
    }

    pub fn next_wallpaper(&mut self) {
        if self.index + 1 < self.paths.len() {
            self.index += 1;
        } else {
            self.shuffle();
        }
    }

    pub fn prev_wallpaper(&mut self) {
        if self.index > 0 {
            self.index -= 1;
        } else {
            self.shuffle()
        }
    }
}

impl Deref for Paths {
    type Target = Vec<String>;

    fn deref(&self) -> &Self::Target {
        &self.paths
    }
}

impl DerefMut for Paths {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.paths
    }
}

impl std::fmt::Debug for Paths {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Paths")
            .field("paths", &self.paths)
            .field("index", &self.index)
            .finish()
    }
}

pub fn get_current() -> anyhow::Result<PathBuf> {
    let output = Command::new("/usr/bin/swww").arg("query").output().unwrap();
    let output_str = str::from_utf8(&output.stdout).unwrap();
    let pattern = Regex::new(r"\/.*\.[\w:]+").unwrap();
    if let Some(matches) = pattern.find(output_str) {
        let path = matches.as_str();
        return Ok(PathBuf::from(path));
    }
    Err(anyhow::anyhow!("[swww query] No valid path found"))
}

pub fn set_wallpaper(paths: &Paths) -> Result<(), Box<(dyn Error + 'static)>> {
    let current = paths.current().unwrap().as_str();
    let _ = Command::new("/usr/bin/swww")
        .arg("img")
        .arg(current)
        .spawn()?;

    Ok(())
}

pub fn get_transition() {
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
