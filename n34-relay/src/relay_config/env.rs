// n34-relay - A nostr GRASP relay implementation
// Copyright (C) 2025 Awiteb <a@4rs.nl>
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published
// by the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program. If not, see <https://gnu.org/licenses/agpl-3.0>.

use std::{borrow::Cow, collections::HashMap, convert::Infallible};

use config::Config;
use const_format::concatcp;
use convert_case::{Case, Casing};
use serde::{Deserialize, Serialize};

use super::{
    CoreConfig,
    LmdbConfig,
    NetworkConfig,
    Nip11Config,
    Nip11Limitation,
    RatelimitConfig,
    RelayConfig,
};
use crate::{
    errors::RelayError,
    relay_config::{GraspConfig, PluginsConfig, RhaiPluginsConfig},
};

/// Prefix used for all environment variables.
pub const ENV_PREFIX: &str = "N34_RELAY";

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
enum ValueKind {
    String(String),
    Bool(bool),
    Array(Vec<ValueKind>),
    U64(u64),
}

/// Parse a string value into a [`ValueKind`]
fn parse_value(value: Cow<'_, str>, is_array: bool) -> ValueKind {
    if is_array {
        return ValueKind::Array(
            value
                .split(',')
                .map(|v| parse_value(Cow::Borrowed(v), false))
                .collect(),
        );
    }

    if let Ok(parsed) = value.parse::<u64>() {
        ValueKind::U64(parsed)
    } else if ["yes", "true"].contains(&value.trim()) {
        ValueKind::Bool(true)
    } else if ["no", "false"].contains(&value.trim()) {
        ValueKind::Bool(false)
    } else {
        ValueKind::String(value.into_owned())
    }
}

/// Builds a configuration for a type using environment variables.
pub trait ConfigFromEnv<const N: usize> {
    /// The environment variables prefix
    const ENV_PREFIX: &str;
    /// The key of the type in snake case. If not empty, it must end with a dot.
    const KEY: &str;
    /// Contains a list of type keys in snake case, where the boolean indicates
    /// whether the value should be parsed as an array.
    const KEYS: [(&'static str, bool); N];
    /// Error type that can be returned by the `from_env` function.
    type Err;

    fn from_env() -> Result<Config, Self::Err> {
        let mut map: HashMap<String, ValueKind> = HashMap::new();

        for (key, is_array) in Self::KEYS {
            if let Ok(value) = std::env::var(format!(
                "{}_{}",
                Self::ENV_PREFIX,
                key.from_case(Case::Snake).to_case(Case::Constant)
            )) {
                map.insert(
                    format!("{}{key}", Self::KEY),
                    parse_value(Cow::Owned(value), is_array),
                );
            }
        }

        Ok(Config::try_from(&map).expect("Can't fail"))
    }
}

impl ConfigFromEnv<2> for NetworkConfig {
    type Err = Infallible;

    const ENV_PREFIX: &str = concatcp!(ENV_PREFIX, "_NET");
    const KEY: &str = "net.";
    const KEYS: [(&'static str, bool); 2] = [("ip", false), ("port", false)];
}

impl ConfigFromEnv<4> for LmdbConfig {
    type Err = Infallible;

    const ENV_PREFIX: &str = concatcp!(ENV_PREFIX, "_LMDB");
    const KEY: &str = "lmdb.";
    const KEYS: [(&'static str, bool); 4] = [
        ("dir", false),
        ("map_size", false),
        ("max_readers", false),
        ("additional_dbs", false),
    ];
}

impl ConfigFromEnv<2> for RatelimitConfig {
    type Err = Infallible;

    const ENV_PREFIX: &str = concatcp!(ENV_PREFIX, "_RATELIMIT");
    const KEY: &str = "ratelimit.";
    const KEYS: [(&'static str, bool); 2] = [("max_queries", false), ("events_per_minute", false)];
}

impl ConfigFromEnv<8> for Nip11Config {
    type Err = Infallible;

    const ENV_PREFIX: &str = concatcp!(ENV_PREFIX, "_NIP11");
    const KEY: &str = "nip11.";
    const KEYS: [(&'static str, bool); 8] = [
        ("name", false),
        ("description", false),
        ("banner", false),
        ("icon", false),
        ("admin", false),
        ("contact", false),
        ("privacy_policy", false),
        ("terms_of_service", false),
    ];
}

impl ConfigFromEnv<2> for Nip11Limitation {
    type Err = Infallible;

    const ENV_PREFIX: &str = concatcp!(Nip11Config::ENV_PREFIX, "_LIMITATION");
    const KEY: &str = "nip11.limitation.";
    const KEYS: [(&'static str, bool); 2] = [("payment_required", false), ("payments_url", false)];
}

impl ConfigFromEnv<2> for RhaiPluginsConfig {
    type Err = Infallible;

    const ENV_PREFIX: &str = concatcp!(ENV_PREFIX, "_RHAI");
    const KEY: &str = "plugins.rhai.";
    const KEYS: [(&'static str, bool); 2] = [("workers", false), ("plugins", true)];
}

impl ConfigFromEnv<1> for PluginsConfig {
    type Err = Infallible;

    const ENV_PREFIX: &str = concatcp!(ENV_PREFIX, "_PLUGINS");
    const KEY: &str = "plugins.";
    const KEYS: [(&'static str, bool); 1] = [("grpc", true)];
}

impl ConfigFromEnv<4> for GraspConfig {
    type Err = Infallible;

    const ENV_PREFIX: &str = concatcp!(ENV_PREFIX, "_GRASP");
    const KEY: &str = "grasp.";
    const KEYS: [(&'static str, bool); 4] = [
        ("enable", false),
        ("git_path", false),
        ("max_reqs", false),
        ("req_timeout", false),
    ];
}

impl ConfigFromEnv<13> for CoreConfig {
    type Err = Infallible;

    const ENV_PREFIX: &str = ENV_PREFIX;
    const KEY: &str = "";
    const KEYS: [(&'static str, bool); 13] = [
        ("domain", false),
        ("nip42", false),
        ("max_connections", false),
        ("min_pow", false),
        ("max_event_size", false),
        ("max_limit", false),
        ("default_limit", false),
        ("max_subid_length", false),
        ("whitelist", true),
        ("blacklist", true),
        ("admins", true),
        ("allowed_kinds", true),
        ("disallowed_kinds", true),
    ];
}

impl ConfigFromEnv<0> for RelayConfig {
    type Err = RelayError;

    // Constants defined with empty values as the `from_env` function is not
    // implemented by the trait.
    const ENV_PREFIX: &str = "";
    const KEY: &str = "";
    const KEYS: [(&'static str, bool); 0] = [];

    fn from_env() -> Result<Config, Self::Err> {
        Config::builder()
            .add_source(NetworkConfig::from_env().expect("Infallible"))
            .add_source(LmdbConfig::from_env().expect("Infallible"))
            .add_source(RatelimitConfig::from_env().expect("Infallible"))
            .add_source(Nip11Config::from_env().expect("Infallible"))
            .add_source(Nip11Limitation::from_env().expect("Infallible"))
            .add_source(RhaiPluginsConfig::from_env().expect("Infallible"))
            .add_source(PluginsConfig::from_env().expect("Infallible"))
            .add_source(GraspConfig::from_env().expect("Infallible"))
            .add_source(CoreConfig::from_env().expect("Infallible"))
            .build()
            .map_err(|err| RelayError::Config(err.to_string()))
    }
}

#[cfg(test)]
mod test {
    use std::env::{self, temp_dir};

    use nostr::key::PublicKey;

    use super::*;
    use crate::ext_traits::RwlockVecExt;

    fn set_base_dir() {
        unsafe {
            env::set_var("N34_RELAY_BASE_DIR", temp_dir().join("n34-test"));
        }
    }

    #[test]
    fn core() {
        set_base_dir();

        unsafe {
            env::set_var("N34_RELAY_NIP42", "yes");
            env::set_var("N34_RELAY_MAX_CONNECTIONS", "123");
            env::set_var("N34_RELAY_MIN_POW", "10");
            env::set_var("N34_RELAY_MAX_EVENT_SIZE", "150000");
        }

        let config: RelayConfig = RelayConfig::from_env().unwrap().try_deserialize().unwrap();
        assert!(config.relay.nip42);
        assert_eq!(config.relay.max_connections, Some(123));
        assert_eq!(config.relay.min_pow, 10);
        assert_eq!(usize::from(config.relay.max_event_size), 150_000);
    }

    #[test]
    fn array() {
        set_base_dir();

        let npub1 =
            PublicKey::parse("npub1rzr6599lsp5fpy4qzytq6nccmpc09d8ekcc9vpemvereyuwp94qs0krtmz")
                .unwrap();
        let npub2 =
            PublicKey::parse("npub1m2a0ht539xwr6evljppgdk6hn73e4p3dn2a23hx0ddt20rs9xsnsfx06cn")
                .unwrap();
        let npub3 =
            PublicKey::parse("npub1wc0d0uuu02ua59h73khxu7trhsafvcvtp4xzqjkwjzvs8pyz0myq2vjxn6")
                .unwrap();

        unsafe {
            env::set_var("N34_RELAY_WHITELIST", format!("{npub1},{npub2},{npub3}"));
            env::set_var("N34_RELAY_BLACKLIST", format!("{npub1},{npub2},{npub3}"));
        }

        let config: RelayConfig = RelayConfig::from_env().unwrap().try_deserialize().unwrap();
        assert!(config.relay.whitelist.contains(&npub1));
        assert!(config.relay.whitelist.contains(&npub2));
        assert!(config.relay.whitelist.contains(&npub3));

        assert!(config.relay.blacklist.contains(&npub1));
        assert!(config.relay.blacklist.contains(&npub2));
        assert!(config.relay.blacklist.contains(&npub3));
    }

    #[test]
    fn nested() {
        set_base_dir();

        unsafe {
            env::set_var("N34_RELAY_NET_IP", "0.0.0.0");
            env::set_var("N34_RELAY_NET_PORT", "8888");
            env::set_var("N34_RELAY_LMDB_MAP_SIZE", "9999");
            env::set_var("N34_RELAY_LMDB_DIR", "/app/lmdb");
        }

        let config: RelayConfig = RelayConfig::from_env().unwrap().try_deserialize().unwrap();
        assert_eq!(config.net.ip.to_string(), "0.0.0.0");
        assert_eq!(config.net.port, 8888);
        assert_eq!(config.lmdb.map_size, 9999);
        assert_eq!(config.lmdb.dir.to_string_lossy(), "/app/lmdb");
    }
}
