use std::collections::{BTreeMap, BTreeSet};

#[cfg(feature = "clap")]
pub mod cli;
#[cfg(feature = "env")]
pub mod envvar;
pub mod error;

#[cfg(feature = "clap")]
pub use cli::{StoreArgs, parse_key_value_clap};
#[cfg(feature = "env")]
pub use envvar::all_from_env_vars;
pub use error::{ConfigError, Result};

const URI_KEY: &str = "uri";

type StoreName = String;
type ConfigKey = String;
type ConfigValue = String;

#[derive(Debug, Clone, Default)]
pub struct StoreConfigCollector {
    pub name_cfg_values: BTreeMap<StoreName, BTreeMap<ConfigKey, Values>>,
}

/// Parse a `key=value` string. Only the first `=` is significant.
pub fn parse_key_value(s: &str) -> Option<(&str, &str)> {
    let (k, v) = s.split_once('=')?;
    let k = k.trim();
    if k.is_empty() {
        return None;
    }
    Some((k, v.trim()))
}

#[derive(Debug, Clone)]
pub struct Values {
    first: ConfigValue,
    last: Option<ConfigValue>,
}

impl Values {
    pub fn new(first: impl Into<ConfigValue>) -> Self {
        Self {
            first: first.into(),
            last: None,
        }
    }

    pub fn push(&mut self, value: impl Into<ConfigValue>) {
        self.last = Some(value.into());
    }

    pub fn first(&self) -> &ConfigValue {
        &self.first
    }

    pub fn into_first(self) -> ConfigValue {
        self.first
    }

    pub fn into_last(self) -> ConfigValue {
        self.last.unwrap_or(self.first)
    }

    pub fn last(&self) -> &ConfigValue {
        self.last.as_ref().unwrap_or(&self.first)
    }

    pub fn choose(self, prefer_first: bool) -> ConfigValue {
        if prefer_first {
            self.first
        } else {
            self.last.unwrap_or(self.first)
        }
    }
}

impl From<ConfigValue> for Values {
    fn from(value: ConfigValue) -> Self {
        Self::new(value)
    }
}

impl StoreConfigCollector {
    pub fn add_config(&mut self, name: impl AsRef<str>, config: StoreConfig) -> usize {
        let name = name.as_ref();
        let count = config.options.len() + 1;
        self.add(name, URI_KEY, config.uri);
        for (key, value) in config.options {
            self.add(name, key, value);
        }
        count
    }

    pub fn add_many(
        &mut self,
        name: impl AsRef<str>,
        configs: impl IntoIterator<Item = (impl AsRef<str>, impl Into<String>)>,
    ) -> usize {
        let mut count = 0;
        for (k, v) in configs {
            self.add(name.as_ref(), k, v);
            count += 1;
        }
        count
    }

    pub fn add(&mut self, name: impl AsRef<str>, key: impl AsRef<str>, value: impl Into<String>) {
        use std::collections::btree_map::Entry;
        let name = name.as_ref();
        let key = key.as_ref();
        let value = value.into();

        match self
            .name_cfg_values
            .entry(name.into())
            .or_default()
            .entry(key.into())
        {
            Entry::Vacant(vacant_entry) => {
                vacant_entry.insert(value.into());
            }
            Entry::Occupied(occupied_entry) => {
                occupied_entry.into_mut().push(value);
            }
        }
    }

    /// Count the number of fields added and updated.
    pub fn update(&mut self, other: StoreConfigCollector) -> (usize, usize) {
        use std::collections::btree_map::Entry;
        let mut added = 0;
        let mut updated = 0;
        for (name, key_values) in other.name_cfg_values {
            let entry = self.name_cfg_values.entry(name).or_default();
            for (key, values) in key_values {
                match entry.entry(key) {
                    Entry::Vacant(vacant_entry) => {
                        added += 1;
                        vacant_entry.insert(values);
                    }
                    Entry::Occupied(occupied_entry) => {
                        updated += 1;
                        occupied_entry.into_mut().push(values.into_last());
                    }
                }
            }
        }
        (added, updated)
    }

    /// Convert into a map of `StoreConfig`s, and a set of names for which no URI was found.
    ///
    /// If `prefer_first` is true, the first value added for each key is used.
    /// If false, the last key is used instead.
    pub fn into_configs(
        self,
        prefer_first: bool,
    ) -> (BTreeMap<StoreName, StoreConfig>, BTreeSet<StoreName>) {
        let mut missing_uris = BTreeSet::new();
        let mut configs = BTreeMap::new();
        for (name, mut key_values) in self.name_cfg_values {
            let Some(uri) = key_values.remove(URI_KEY) else {
                missing_uris.insert(name);
                continue;
            };
            let mut config = StoreConfig::new(uri.choose(prefer_first));
            for (key, values) in key_values {
                config.options.insert(key, values.choose(prefer_first));
            }
            configs.insert(name, config);
        }
        (configs, missing_uris)
    }
}

/// A URI plus additional options from which a store can be built.
///
/// With the `serde` feature this can be (de)serialized, e.g.
///
/// ```toml
/// uri = "s3://my-bucket/prefix"
/// region = "us-east-1"
/// ```
///
/// `options` may be omitted.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct StoreConfig {
    pub uri: String,
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "BTreeMap::is_empty", flatten)
    )]
    pub options: BTreeMap<String, String>,
}

impl StoreConfig {
    pub fn new(uri: impl Into<String>) -> Self {
        Self {
            uri: uri.into(),
            options: BTreeMap::new(),
        }
    }

    /// Convert into a `(uri, options)` tuple suitable for passing to [`opendal::Operator::from_uri`](https://docs.rs/opendal/latest/opendal/struct.Operator.html#method.from_uri)
    /// or [`object_store::parse_url_opts`](https://docs.rs/object_store/latest/object_store/fn.parse_url_opts.html).
    pub fn into_tuple(self) -> (String, BTreeMap<String, String>) {
        (self.uri, self.options)
    }

    /// Add an option, replacing any existing value for that key.
    pub fn with_option(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.options.insert(key.into(), value.into());
        self
    }

    /// Add options, replacing existing values for the same keys.
    pub fn extend_options<K: Into<String>, V: Into<String>>(
        &mut self,
        options: impl IntoIterator<Item = (K, V)>,
    ) {
        self.options
            .extend(options.into_iter().map(|(k, v)| (k.into(), v.into())));
    }

    /// Overlay `other` on top of `self`. The `other` URI and options take precedence.
    pub fn merge(&mut self, other: StoreConfig) -> &mut Self {
        self.uri = other.uri;
        self.options.extend(other.options);
        self
    }
}
