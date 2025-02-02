use std::{
    error::Error,
    fmt::{self, Display},
    io,
    path::PathBuf,
};

#[derive(Debug)]
pub enum DirError {
    DoesNotExist(PathBuf),
    InvalidPath(PathBuf),
    IoError(io::Error),
    MissingVar(String),
}

impl Display for DirError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DoesNotExist(p) => writeln!(f, "Directory does not exist: {:?}", p),
            Self::InvalidPath(p) => writeln!(f, "Invalid path: {:?}", p),
            Self::IoError(io_err) => writeln!(f, "I/O error: {}", io_err),
            Self::MissingVar(var) => writeln!(f, "{var} environment variable is not set"),
        }
    }
}

impl Error for DirError {}
