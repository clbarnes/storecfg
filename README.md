# opendal-config

Helper for applications which need to define Apache OpenDAL™ services and layers.

## Usage

See [`example.rs`](./examples/example.rs) for a concrete example.

The goal is to produce an [`OperatorConfig`] for one or more services.
This can be converted into a `(url, options_map)` tuple with [`OperatorConfig::into_tuple`],
which can then be passed in to [`opendal::Operator::from_uri`].

The [`OperatorConfigCollector`] gathers configuration from different sources.
Each operator has a name (with a single anonymous operator represented by an empty string),
must have a config key called `uri`,
and may have any number of additional config key-value pairs.

### Configuration files

Requires the `serde` feature (default).

The [`OperatorConfig`] implements `serde::{Serialize, Deserialize}` from a map of string to string.
This means they can be deserialised from sources like JSON or TOML:

```rust ignore
use opendal_config::OperatorConfig;

let json_str = r#"{
  "my_service": {
    "url": "https://my_store.com",
    "username": "alice",
    "password": "Hunter2"
  }
}"#;

let o1: BTreeMap<String, OperatorConfig> = serde_json::from_str(json_str).unwrap();

let toml_str = r#"
[my_service]
url = "https://my_store.com"
username = "alice"
password = "Hunter2"
"#;

let o2: BTreeMap<String, OperatorConfig> = toml::from_str(toml_str).unwrap();
```

### Environment variables

Requires the `env` feature (default).

Environment variables are, on some platforms, case-insensitive.
It is recommended that,
if you intend to configure any of your services with environment variables,
they should be all be lower-case.

[`OperatorConfig::add_from_env`] takes a string which prefixes all configuration (e.g. `"OPENDAL_SERVICE"`).
The key of the configuration item is prefixed with a double-underscore `__`.
To configure the anonymous operator, simply use the prefix, a double underscore, and the key (`OPENDAL_SERVICE__URI="https://..."`).
Otherwise, add an underscore and the name operator name after the prefix.

The operator name and configuration key will both be normalised to lower case.

For example, given the prefix `"OPENDAL_SERVICE"`,
the environment variable key-value pair `OPENDAL_SERVICE_SOME_OP__USER_NAME="alice"`
would configure the operator `some_op`, adding the key `user_name` with the value `alice`.

### CLI arguments

CLIs should take URIs and optionally some number of key-value configuration pairs.

[`parse_key_value`] splits a `"key=value"` string.

#### Clap

Requires the `clap` feature.

[`cli::parse_key_value_clap`] can be used as a clap `value_parser`.

[`cli::OperatorArgs`] is a positional argument and a `-o`/`--option` option which can be given multiple times and takes options in the `key=value` form.
This can be added to a `clap::Parser` struct with `#[command(flatten)]`.

```rust ignore
use clap::Parser;

#[derive(Debug, Parser)]
struct Cli {
    #[command(flatten)]
    store: opendal_config::cli::OperatorArgs,
}

// `my_cli https://mystore.com -o a=b -o c=d`
let args = Cli::parse();
let conf = opendal_config::OperatorConfig::from(args.store);
// OperatorConfig {
//   uri: "https://mystore.com",
//   options: {
//     "a": "b",
//     "c": "d"
//   }
// }
```
