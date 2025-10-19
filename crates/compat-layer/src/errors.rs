use thiserror::Error;

/// Represents an error that can occur during the parsing or migration of a Klipper config.
#[derive(Error, Debug, PartialEq)]
pub enum MigrationError {
    #[error("Parsing error on line {0}: {1}")]
    ParseError(usize, String),

    #[error("Missing required section: [{0}]")]
    MissingSection(String),

    #[error("Missing required key '{key}' in section [{section}]")]
    MissingKey { section: String, key: String },

    #[error("Invalid value for key '{key}' in section [{section}]: {message}")]
    InvalidValue {
        section: String,
        key: String,
        message: String,
    },

    #[error("Unsupported kinematics type: {0}")]
    UnsupportedKinematics(String),

    #[error("An I/O error occurred")]
    IoError(#[from] std::io::Error),
}
