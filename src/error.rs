use std::{
    error::Error,
    fmt::{self, Display},
};

#[derive(Debug)]
pub enum ParseTransitionFlavourError {
    InvalidFlavour(String),
}

impl Display for ParseTransitionFlavourError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidFlavour(s) => write!(f, "Invalid transition type: {}", s),
        }
    }
}

impl Error for ParseTransitionFlavourError {}
