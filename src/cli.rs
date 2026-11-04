//! Command-line argument support via `clap`.

use crate::{StoreConfig, parse_key_value};

/// Command-line arguments for configuring an .
///
/// Flatten into your own parser:
///
/// ```ignore
/// #[derive(clap::Parser)]
/// struct Cli {
///     #[command(flatten)]
///     store: storecfg::StoreArgs,
/// }
/// ```
///
/// Usage: `my-cli https://mystore.com -o key=value -o anotherkey=anothervalue`
#[derive(Debug, Clone, PartialEq, Eq, clap::Args)]
pub struct StoreArgs {
    /// URI of the storage backend.
    pub uri: String,
    /// Additional backend options as 'key=value'; may be repeated.
    #[arg(short = 'o', long = "options", value_name = "KEY=VALUE", value_parser = parse_key_value_clap)]
    pub options: Vec<(String, String)>,
}

/// [`parse_key_value`] with a `String` error, for use as a clap value parser.
pub fn parse_key_value_clap(s: &str) -> Result<(String, String), String> {
    match parse_key_value(s) {
        Some((k, v)) => Ok((k.to_string(), v.to_string())),
        None => Err(format!("invalid option {s:?}; expected KEY=VALUE")),
    }
}

impl From<StoreArgs> for StoreConfig {
    fn from(args: StoreArgs) -> Self {
        let mut c = StoreConfig::new(args.uri);
        c.extend_options(args.options);
        c
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cli() {
        use clap::Parser;
        #[derive(Parser)]
        struct Cli {
            #[command(flatten)]
            store: StoreArgs,
        }
        let cli = Cli::parse_from(["x", "https://mystore.com", "-o", "k=v", "-o", "k2=v2"]);
        let c: StoreConfig = cli.store.into();
        assert_eq!(
            c,
            StoreConfig::new("https://mystore.com")
                .with_option("k", "v")
                .with_option("k2", "v2")
        );
    }
}
