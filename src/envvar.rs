//! Reading configuration from environment variables.

use crate::{ConfigError, Result, StoreConfigCollector};

pub(crate) const NAME_SUFFIX_SEP: &str = "__";

/// Tuple of `(name, key, value)` for each environment variable matching the given prefix.
///
/// Name is the part of the variable name after the prefix and before the first double underscore, lowercased.
/// Key is the part of the variable name after the first double underscore, lowercased.
/// Value is the value of the environment variable.
pub fn all_from_env_vars<K, V>(
    prefix: &str,
    vars: impl IntoIterator<Item = (K, V)>,
) -> impl Iterator<Item = (String, String, String)>
where
    K: AsRef<str>,
    V: Into<String>,
{
    vars.into_iter().filter_map(|(k, v)| {
        let kref = k.as_ref();
        let (leader, key) = kref.rsplit_once(NAME_SUFFIX_SEP)?;
        if leader.len() < prefix.len() {
            return None;
        }

        if !&leader[..prefix.len()].eq_ignore_ascii_case(prefix) {
            return None;
        }
        let name = &leader[prefix.len()..];
        log::debug!("Raw name: {name}, key: {key}");
        if !name.is_empty() && !name.starts_with('_') {
            return None;
        }
        let name = name.trim_start_matches('_').to_lowercase();
        let out = (name, key.to_lowercase(), v.into());
        log::debug!("Parsed env var: {out:?}");
        Some(out)
    })
}

impl StoreConfigCollector {
    /// Read all services configured in the process environment and add them to this collector.
    pub fn add_from_env(&mut self, prefix: &str) -> Result<usize, ConfigError> {
        let env_vars = std::env::vars();
        let mut count = 0;
        for (name, key, value) in all_from_env_vars(prefix, env_vars) {
            self.add(name, key, value);
            count += 1;
        }
        Ok(count)
    }
}

#[derive(Debug, Clone)]
pub enum NameProblem {
    NonAlphanumeric,
    Uppercase,
}

impl std::fmt::Display for NameProblem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NameProblem::NonAlphanumeric => write!(f, "name contains non-alphanumeric characters"),
            NameProblem::Uppercase => write!(f, "name contains uppercase characters"),
        }
    }
}

impl std::error::Error for NameProblem {}

impl NameProblem {
    pub fn maybe_new(name: &str) -> Option<Self> {
        for c in name.chars() {
            if c.is_ascii_uppercase() {
                return Some(NameProblem::Uppercase);
            }
            if !c.is_alphanumeric() && c != '_' {
                return Some(NameProblem::NonAlphanumeric);
            }
        }
        None
    }
}
