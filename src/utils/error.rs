use std::fmt;

#[derive(Debug)]
#[allow(dead_code)]
pub enum CliError {
    NotFound {
        entity: String,
        identifier: String,
    },
    MultipleMatches {
        entity: String,
        identifier: String,
        candidates: Vec<String>,
    },
    InvalidParameter {
        param: String,
        reason: String,
    },
    RequiresParameter {
        flag: String,
        required: String,
    },
    MutuallyExclusive {
        flag_a: String,
        flag_b: String,
    },
    GraphqlError(String),
    HttpError(reqwest::Error),
    AuthError(String),
    IoError(std::io::Error),
    FileTooLarge {
        path: String,
        size: u64,
        max: u64,
    },
    Other(String),
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotFound {
                entity,
                identifier,
            } => write!(f, "{entity} \"{identifier}\" not found"),
            Self::MultipleMatches {
                entity,
                identifier,
                candidates,
            } => write!(
                f,
                "Multiple {entity} matches for \"{identifier}\": {}",
                candidates.join(", ")
            ),
            Self::InvalidParameter { param, reason } => {
                write!(f, "Invalid parameter \"{param}\": {reason}")
            }
            Self::RequiresParameter { flag, required } => {
                write!(f, "Flag \"{flag}\" requires \"{required}\"")
            }
            Self::MutuallyExclusive { flag_a, flag_b } => {
                write!(f, "Flags \"{flag_a}\" and \"{flag_b}\" are mutually exclusive")
            }
            Self::GraphqlError(msg) => write!(f, "{msg}"),
            Self::HttpError(err) => write!(f, "HTTP error: {err}"),
            Self::AuthError(msg) => write!(f, "{msg}"),
            Self::IoError(err) => write!(f, "IO error: {err}"),
            Self::FileTooLarge { path, size, max } => {
                write!(
                    f,
                    "File \"{path}\" is too large ({size} bytes, max {max} bytes)"
                )
            }
            Self::Other(msg) => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for CliError {}

impl From<reqwest::Error> for CliError {
    fn from(err: reqwest::Error) -> Self {
        Self::HttpError(err)
    }
}

impl From<std::io::Error> for CliError {
    fn from(err: std::io::Error) -> Self {
        Self::IoError(err)
    }
}

impl From<serde_json::Error> for CliError {
    fn from(err: serde_json::Error) -> Self {
        Self::Other(format!("JSON error: {err}"))
    }
}
