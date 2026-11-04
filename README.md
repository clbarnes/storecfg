# storecfg

Helper for applications which need to define [Apache OpenDAL™](https://crates.io/crates/opendal) operators or [object_store](https://crates.io/crates/object_store) object stores from configuration.

## Usage

See [`example.rs`](./examples/example.rs) for a concrete example.

The goal is to produce a [`StoreConfig`] for one or more services.
This can be converted into a `(url, options_map)` tuple with [`StoreConfig::into_tuple`],
which can then be passed in to [`opendal::Operator::from_uri`](https://docs.rs/opendal/latest/opendal/struct.Operator.html#method.from_uri)
or [`object_store::parse_url_opts`](https://docs.rs/object_store/latest/object_store/fn.parse_url_opts.html).

The [`StoreConfigCollector`] gathers configuration from different sources.
Each store has a name (with a single anonymous store represented by an empty string),
must have a config key called `uri`,
and may have any number of additional config key-value pairs.

### Configuration files

Requires the `serde` feature (default).

The [`StoreConfig`] implements `serde::{Serialize, Deserialize}` from a map of string to string.
This means they can be deserialised from sources like JSON or TOML:

```rust ignore
use storecfg::StoreConfig;

let json_str = r#"{
  "my_service": {
    "uri": "https://my_store.com",
    "username": "alice",
    "password": "Hunter2"
  }
}"#;

let o1: BTreeMap<String, StoreConfig> = serde_json::from_str(json_str).unwrap();

let toml_str = r#"
[my_service]
url = "https://my_store.com"
username = "alice"
password = "Hunter2"
"#;

let o2: BTreeMap<String, StoreConfig> = toml::from_str(toml_str).unwrap();
```

Note that if you are using a config file in combination with other sources (arguments, environment variables),
you probably want to deserialise into an `impl IntoIterator<Item = (String, String)>` (e.g. a HashMap),
in case the required `uri` field is provided by another source.

### Environment variables

Requires the `env` feature (default).

Environment variables are, on some platforms, case-insensitive.
It is recommended that,
if you intend to configure any of your services with environment variables,
they should be all be lower-case.

[`StoreConfig::add_from_env`] takes a string which prefixes all configuration (e.g. `"STORAGE_"`).
The key of the configuration item is prefixed with a double-underscore `__`.
To configure the anonymous store, simply use the prefix, a double underscore, and the key (`STORAGE__URI="https://..."`).
Otherwise, add an underscore and the store name after the prefix.

The store name and configuration key will both be normalised to lower case when collected from environment variables.

For example, given the prefix `"STORAGE"`,
the environment variable key-value pair `STORAGE_SOME_STORE__USER_NAME="alice"`
would configure the store `some_store`, adding the key `user_name` with the value `alice`.

### CLI arguments

CLIs should take URIs and optionally some number of key-value configuration pairs.

[`parse_key_value`] splits a `"key=value"` string.

#### Clap

Requires the `clap` feature.

[`cli::parse_key_value_clap`] can be used as a clap `value_parser`.

[`cli::StoreArgs`] is a positional argument and a `-o`/`--option` option which can be given multiple times and takes options in the `key=value` form.
This can be added to a `clap::Parser` struct with `#[command(flatten)]`.

```rust ignore
use clap::Parser;

#[derive(Debug, Parser)]
struct Cli {
    #[command(flatten)]
    store: storecfg::cli::StoreArgs,
}

// `my_cli https://mystore.com -o a=b -o c=d`
let args = Cli::parse();
let conf = storecfg::StoreConfig::from(args.store);
// StoreConfig {
//   uri: "https://mystore.com",
//   options: {
//     "a": "b",
//     "c": "d"
//   }
// }
```

## Configuration keys

Other than `"uri"`, the keys you need to set and the form their values need to take (other than being string-like) will vary based on the store and the implementation.

`object_store`s' configuration are documented in the [builder structs](https://docs.rs/object_store/latest/object_store/#available-objectstore-implementations).
`opendal` operators' configuration are documented [on the website](https://opendal.apache.org/services/).
In both cases, 3rd party crates can provide additional backends which need their own configuration;
you will need to compile support for them separately.
