use thiserror::Error;

/// Represents an error that can occur during the parsing or migration of a Klipper config.
#[derive(Error, Debug, PartialEq)]
pub enum MigrationError {
    /// An error occurred while parsing the configuration file.
    #[error("Parsing error on line {0}: {1}")]
    ParseError(usize, String),

    /// A required section was not found in the configuration.
    #[error("Missing required section: [{0}]")]
    MissingSection(String),

    /// A required key was not found within a specific section.
    #[error("Missing required key '{key}' in section [{section}]")]
    MissingKey { section: String, key: String },

    /// A key has an invalid or unparseable value.
    #[error("Invalid value for key '{key}' in section [{section}]: {message}")]
    InvalidValue {
        section: String,
        key: String,
        message: String,
    },

    /// The specified kinematics type is not supported by the migrator.
    #[error("Unsupported kinematics type: {0}")]
    UnsupportedKinematics(String),

    /// An underlying I/O error occurred while reading the file.
    #[error("An I/O error occurred")]
    IoError(#[from] std::io::Error),
}
