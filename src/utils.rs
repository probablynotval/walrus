use env_logger::Target;
use log::LevelFilter;
use std::error::Error;

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
