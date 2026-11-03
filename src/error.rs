//! Error and result types.

use std::fmt;

/// Errors produced while reading configuration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigError {
    /// A `key=value` option could not be parsed.
    InvalidOption(String),
    /// No URI was found for the named service in the environment.
    MissingUri(String),
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::InvalidOption(s) => {
                write!(f, "invalid option {s:?}; expected KEY=VALUE")
            }
            ConfigError::MissingUri(name) => {
                write!(f, "no URI variable found for service: {name}")
            }
        }
    }
}

impl std::error::Error for ConfigError {}

/// Result type defaulting to [`ConfigError`].
pub type Result<T, E = ConfigError> = std::result::Result<T, E>;
