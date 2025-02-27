use std::fmt::{self, Display};

use crate::config::TransitionFlavour;

type Bezier = [f32; 4];

pub struct Pos {
    pub x: f32,
    pub y: f32,
}

pub struct WaveSize {
    pub width: u32,
    pub height: u32,
}

pub enum FilterMethod {
    Nearest,
    Bilinear,
    CatmullRom,
    Mitchell,
    Lanczos3,
}

pub enum ResizeMethod {
    No,
    Crop,
    Fit,
}

impl Display for ResizeMethod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::No => write!(f, "no"),
            Self::Fit => write!(f, "fit"),
            Self::Crop => write!(f, "crop"),
        }
    }
}

impl Display for FilterMethod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Nearest => write!(f, "Nearest"),
            Self::Bilinear => write!(f, "Bilinear"),
            Self::CatmullRom => write!(f, "CatmullRom"),
            Self::Mitchell => write!(f, "Mitchell"),
            Self::Lanczos3 => write!(f, "Lanczos3"),
        }
    }
}

enum ImgArg {
    Resize(ResizeMethod),
    FillColor(String), // Hex color RRGGBB
    Filter(FilterMethod),
    TransitionType(TransitionFlavour),
    TransitionStep(u8),
    TransitionDuration(f64),
    TransitionFps(u32),
    TransitionBezier(Bezier),
    TransitionAngle(f32),     // For: Wipe, Wave
    TransitionPos(Pos),       // For: Grow, Outer
    TransitionWave(WaveSize), // For: Wave
}

impl ImgArg {
    fn to_args(&self) -> Vec<String> {
        match self {
            Self::Resize(resize) => vec!["--resize".into(), resize.to_string()],
            Self::FillColor(color) => vec!["--fill-color".into(), color.to_string()],
            Self::Filter(filter) => vec!["--filter".into(), filter.to_string()],
            Self::TransitionType(flavour) => vec!["--transition-type".into(), flavour.to_string()],

            Self::TransitionStep(step) => vec!["--transition-step".into(), step.to_string()],
            Self::TransitionDuration(duration) => {
                vec!["--transition-duration".into(), duration.to_string()]
            }
            Self::TransitionFps(fps) => vec!["--transition-fps".into(), fps.to_string()],
            Self::TransitionBezier(bezier) => {
                vec![
                    "--transition-bezier".into(),
                    format!("{},{},{},{}", bezier[0], bezier[1], bezier[2], bezier[3]),
                ]
            }
            Self::TransitionAngle(angle) => vec!["--transition-angle".into(), angle.to_string()],
            Self::TransitionPos(pos) => {
                vec!["--transition-pos".into(), format!("{},{}", pos.x, pos.y)]
            }
            Self::TransitionWave(size) => vec![
                "--transition-wave".into(),
                format!("{},{}", size.width, size.height),
            ],
        }
    }
}

#[derive(Default, Debug)]
pub struct TransitionArgBuilder {
    args: Vec<Vec<String>>,
}

impl TransitionArgBuilder {
    pub fn new() -> Self {
        let init = vec!["img".to_string()];
        let args = vec![init];
        Self { args }
    }

    pub fn build(self) -> Vec<String> {
        self.args.into_iter().flatten().collect()
    }

    pub fn with_resize(mut self, resize: ResizeMethod) -> Self {
        let arg = ImgArg::Resize(resize).to_args();
        self.args.push(arg);
        self
    }

    pub fn with_fill(mut self, color: String) -> Self {
        let arg = ImgArg::FillColor(color).to_args();
        self.args.push(arg);
        self
    }

    pub fn with_filter(mut self, filter: FilterMethod) -> Self {
        let arg = ImgArg::Filter(filter).to_args();
        self.args.push(arg);
        self
    }

    pub fn with_transition(mut self, flavour: &TransitionFlavour) -> Self {
        let arg = ImgArg::TransitionType(flavour.clone()).to_args();
        self.args.push(arg);
        self
    }

    pub fn with_step(mut self, step: u8) -> Self {
        let arg = ImgArg::TransitionStep(step).to_args();
        self.args.push(arg);
        self
    }

    pub fn with_duration(mut self, duration: f64) -> Self {
        let arg = ImgArg::TransitionDuration(duration).to_args();
        self.args.push(arg);
        self
    }

    pub fn with_fps(mut self, fps: u32) -> Self {
        let arg = ImgArg::TransitionFps(fps).to_args();
        self.args.push(arg);
        self
    }

    pub fn with_bezier(mut self, bezier: Bezier) -> Self {
        let arg = ImgArg::TransitionBezier(bezier).to_args();
        self.args.push(arg);
        self
    }

    pub fn with_angle(mut self, angle: f32) -> Self {
        let arg = ImgArg::TransitionAngle(angle).to_args();
        self.args.push(arg);
        self
    }

    pub fn with_pos(mut self, pos: Pos) -> Self {
        let arg = ImgArg::TransitionPos(pos).to_args();
        self.args.push(arg);
        self
    }

    pub fn with_wave(mut self, wave: WaveSize) -> Self {
        let arg = ImgArg::TransitionWave(wave).to_args();
        self.args.push(arg);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all() {
        let (builder, expected) = test_parsing();

        assert_eq!(builder.build(), expected);

        let (builder, expected) = test_everything();

        assert_eq!(builder.build(), expected);
    }

    fn test_parsing() -> (TransitionArgBuilder, Vec<String>) {
        let builder = TransitionArgBuilder::new()
            .with_transition(&TransitionFlavour::Wave)
            .with_resize(ResizeMethod::Fit)
            .with_filter(FilterMethod::Nearest);

        (
            builder,
            vec![
                "img".into(),
                "--transition-type".into(),
                "wave".into(),
                "--resize".into(),
                "fit".into(),
                "--filter".into(),
                "Nearest".into(),
            ],
        )
    }

    fn test_everything() -> (TransitionArgBuilder, Vec<String>) {
        let builder = TransitionArgBuilder::new()
            .with_transition(&TransitionFlavour::Wipe)
            .with_duration(1.0)
            .with_step(25)
            .with_fps(420)
            .with_resize(ResizeMethod::Crop)
            .with_bezier([0.0, 0.4, 0.0, 0.6])
            .with_filter(FilterMethod::Nearest)
            .with_fill(String::from("FFFFFF"))
            .with_angle(69.0)
            .with_wave(WaveSize {
                width: 5,
                height: 10,
            })
            .with_pos(Pos { x: 10.0, y: 20.0 });

        (
            builder,
            vec![
                "img".into(),
                "--transition-type".into(),
                "wipe".into(),
                "--transition-duration".into(),
                "1".into(),
                "--transition-step".into(),
                "25".into(),
                "--transition-fps".into(),
                "420".into(),
                "--resize".into(),
                "crop".into(),
                "--transition-bezier".into(),
                "0,0.4,0,0.6".into(),
                "--filter".into(),
                "Nearest".into(),
                "--fill-color".into(),
                "FFFFFF".into(),
                "--transition-angle".into(),
                "69".into(),
                "--transition-wave".into(),
                "5,10".into(),
                "--transition-pos".into(),
                "10,20".into(),
            ],
        )
    }
}
