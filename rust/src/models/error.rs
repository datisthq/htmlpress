use std::fmt;
use std::path::PathBuf;

/// Why a build or the dev server failed.
#[derive(Debug)]
pub enum Error {
    /// A file could not be read, written or listed.
    Io {
        path: PathBuf,
        source: std::io::Error,
    },
    /// `htmlpress.config.yaml` is not valid.
    Config { path: PathBuf, message: String },
    /// A page could not be parsed or rendered.
    Page { path: String, message: String },
    /// The options cannot work, such as an output folder that holds the project.
    Usage(String),
}

impl Error {
    pub(crate) fn io(path: impl Into<PathBuf>) -> impl FnOnce(std::io::Error) -> Self {
        let path = path.into();
        move |source| Self::Io { path, source }
    }

    pub(crate) fn page(path: &str) -> impl FnOnce(String) -> Self {
        let path = path.to_owned();
        move |message| Self::Page { path, message }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io { path, source } => {
                write!(formatter, "{}: {source}", path.display())
            }
            Self::Config { path, message } => {
                write!(formatter, "{}: {message}", path.display())
            }
            Self::Page { path, message } => write!(formatter, "{path}: {message}"),
            Self::Usage(message) => formatter.write_str(message),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io { source, .. } => Some(source),
            _ => None,
        }
    }
}
