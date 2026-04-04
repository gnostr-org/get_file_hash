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

use std::{
    borrow::Cow,
    fs,
    net::IpAddr,
    num::{NonZeroU8, NonZeroU64, NonZeroUsize},
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

use config::{Config, File as FileSource, FileFormat};
use nostr::{event::Kind, key::PublicKey, types::Url};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};

use crate::{
    errors::{RelayError, RelayResult},
    ext_traits::{RwlockOption, RwlockVecExt},
    pathes,
    relay_config::env::ConfigFromEnv,
};

/// Config defaults
pub mod defaults;
/// Struct the config from environment variables
pub mod env;
/// Config parsers
mod parsers;

/// A type for public keys list
type ArcRwVec<T> = Arc<RwLock<Vec<T>>>;

/// Indicates whether serialization is for TOML format.
///
/// Some fields are skipped during deserialization from config file,
/// but should be included when serializing to JSON.
pub static SERIALIZE_TO_TOML: AtomicBool = AtomicBool::new(false);

/// Configuration for the relay network.
#[derive(Debug, Deserialize, Serialize)]
pub struct NetworkConfig {
    /// The IP address the relay will bind to.
    #[serde(default = "defaults::net::ip_addr")]
    pub ip:   IpAddr,
    /// The port the relay will listen on.
    #[serde(default = "defaults::net::port")]
    pub port: u16,
}

/// Configuration for an LMDB database instance.
#[derive(Debug, Deserialize, Serialize)]
pub struct LmdbConfig {
    /// Path to the directory where the database files are stored. Defaults
    /// `/etc/n34-relay/config.toml`
    #[serde(default = "defaults::lmdb::dir")]
    pub dir:            PathBuf,
    /// Maximum size of the memory map (in bytes). Defaults to `32GB` on 64-bit
    /// and `4GB` on 32-bit systems.
    #[serde(default = "defaults::lmdb::map_size")]
    pub map_size:       usize,
    /// Maximum number of concurrent reader slots. Defaults to `126`.
    #[serde(default = "defaults::lmdb::max_readers")]
    pub max_readers:    u32,
    /// Number of extra databases to allocate in addition to the 9 internal
    /// ones. Defaults `0`
    #[serde(default = "defaults::lmdb::additional_dbs")]
    pub additional_dbs: u32,
}

/// Ratelimit config
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RatelimitConfig {
    /// Maximum number of active queries allowed. Defaults 500
    #[serde(default = "defaults::ratelimit::max_queries")]
    pub max_queries:       usize,
    /// Number of events allowed per minute. Defaults 120
    #[serde(default = "defaults::ratelimit::events_per_minute")]
    pub events_per_minute: u32,
}

/// Relay metadata for NIP-11.
#[serde_with::skip_serializing_none]
#[derive(Debug, Deserialize, Serialize)]
pub struct Nip11Config {
    /// Name of the relay.
    #[serde(default, skip_serializing_if = "RwlockOption::is_none")]
    pub name:             RwLock<Option<String>>,
    /// Description of the relay.
    #[serde(default, skip_serializing_if = "RwlockOption::is_none")]
    pub description:      RwLock<Option<String>>,
    /// URL of the relay's banner image.
    #[serde(default, skip_serializing_if = "RwlockOption::is_none")]
    pub banner:           RwLock<Option<Url>>,
    /// URL of the relay's icon image.
    #[serde(default, skip_serializing_if = "RwlockOption::is_none")]
    pub icon:             RwLock<Option<Url>>,
    /// Public key of the relay's administrator.
    pub admin:            Option<PublicKey>,
    /// Alternate contact information for the relay administrator.
    pub contact:          Option<String>,
    /// URL to the relay's privacy policy document.
    pub privacy_policy:   Option<Url>,
    /// URL to the relay's terms of service document.
    pub terms_of_service: Option<Url>,
    /// Relay limitation
    #[serde(default)]
    pub limitation:       Nip11Limitation,

    // The following values are defined by the relay and not customizable by the user.
    /// List of NIPs supported by the relay.
    #[serde(
        skip_deserializing,
        skip_serializing_if = "ser_for_toml",
        default = "defaults::nip11::supported_nips"
    )]
    pub supported_nips:   &'static [u16],
    /// List of supported GRASPs
    #[serde(
        skip_deserializing,
        skip_serializing_if = "ser_for_toml",
        default = "defaults::nip11::supported_grasps"
    )]
    pub supported_grasps: &'static [&'static str],
    /// Name of the relay's software.
    #[serde(
        skip_deserializing,
        skip_serializing_if = "ser_for_toml",
        default = "defaults::nip11::software"
    )]
    pub software:         &'static str,
    /// Version of the relay's software.
    #[serde(
        skip_deserializing,
        skip_serializing_if = "ser_for_toml",
        default = "defaults::nip11::version"
    )]
    pub version:          &'static str,
}

// TODO: Add `max_event_tags` and `max_content_length`
/// Represents limitations and requirements for a relay as specified in NIP-11.
#[derive(Default, Debug, Deserialize, Serialize)]
pub struct Nip11Limitation {
    /// Indicates if payment is required to use the relay.
    #[serde(default)]
    pub payment_required: bool,
    /// URL where payments can be made.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payments_url:     Option<Url>,

    // The following values are defined by the relay and not customizable by the user.
    /// Maximum allowed size in bytes for incoming JSON messages.
    #[serde(skip_deserializing, skip_serializing_if = "ser_for_toml", default)]
    pub max_message_length: usize,
    /// Maximum number of active subscriptions allowed per websocket connection.
    #[serde(skip_deserializing, skip_serializing_if = "ser_for_toml", default)]
    pub max_subscriptions:  usize,
    /// Minimum Proof of Work difficulty required for new events.
    #[serde(skip_deserializing, skip_serializing_if = "ser_for_toml", default)]
    pub min_pow_difficulty: u8,
    /// Indicates if NIP-42 authentication is required before performing any
    /// action.
    #[serde(skip_deserializing, skip_serializing_if = "ser_for_toml", default)]
    pub auth_required:      bool,
    /// Indicates if the relay enforces conditions to accept events.
    #[serde(skip_deserializing, skip_serializing_if = "ser_for_toml", default)]
    pub restricted_writes:  bool,
    /// Limits the maximum number of events a client can fetch from a single
    /// subscription filter.
    #[serde(skip_deserializing, skip_serializing_if = "ser_for_toml", default)]
    pub max_limit:          usize,
    /// Sets the default number of events returned when no limit is specified in
    /// a filter.
    #[serde(skip_deserializing, skip_serializing_if = "ser_for_toml", default)]
    pub default_limit:      usize,
    /// The maximum allowed length for a subscription ID as a string.
    #[serde(skip_deserializing, skip_serializing_if = "ser_for_toml", default)]
    pub max_subid_length:   usize,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RhaiPluginsConfig {
    /// Rhai engine workers. Defaults 3
    #[serde(default = "defaults::rhai::workers")]
    pub workers: NonZeroU8,
    /// Rhai plugins
    #[serde(default)]
    pub plugins: Vec<String>,
}

#[derive(Default, Debug, Deserialize, Serialize)]
pub struct PluginsConfig {
    /// gRPC plugins services
    #[serde(
        serialize_with = "parsers::uris_ser",
        deserialize_with = "parsers::uris_de",
        skip_serializing_if = "Vec::is_empty",
        default
    )]
    pub grpc: Vec<hyper::Uri>,
    /// Rhai plugins
    pub rhai: RhaiPluginsConfig,
}

/// Configuration settings for GRASP, including Git-related options.
#[derive(Debug, Deserialize, Serialize)]
pub struct GraspConfig {
    /// Determines if GRASP is enabled or disabled.
    #[serde(default = "defaults::grasp::enable")]
    pub enable:      bool,
    /// Specifies the path to the Git executable. Defaults to 'git'.
    #[serde(default = "defaults::grasp::git_path")]
    pub git_path:    Cow<'static, str>,
    /// The maximum number of requests a Git server can handle simultaneously.
    /// Default, no limit.
    #[serde(default)]
    pub max_reqs:    Option<NonZeroUsize>,
    /// A timeout for git Git server requests in seconds. If the request
    /// exceeded the timeout it will aborted.
    #[serde(default)]
    pub req_timeout: Option<NonZeroU64>,

    /// Repositories path
    #[serde(skip, default = "pathes::grasp_repos")]
    pub repos_path: PathBuf,
}

/// Core relay-specific configs.
#[derive(Debug, Deserialize, Serialize)]
pub struct CoreConfig {
    /// The domain of the relay, excluding the protocol. Example: 'example.com'
    /// or 'relay.example.net'.
    #[serde(default)]
    pub domain:           String, // TODO: Check that it's without a protocol
    /// Whether NIP42 authentication is required. Default is `false`.
    #[serde(default = "defaults::relay::nip42")]
    pub nip42:            bool,
    /// Maximum number of connections allowed. Defaults to no limit.
    #[serde(default)]
    pub max_connections:  Option<usize>,
    /// Minimum Proof of Work difficulty. Defaults to `0`.
    #[serde(default = "defaults::relay::min_pow")]
    pub min_pow:          u8,
    /// Maximum size of an event in bytes. Default is 150KB.
    #[serde(default = "defaults::relay::max_event_size")]
    pub max_event_size:   NonZeroUsize,
    /// Limits the maximum number of events a client can fetch from a single
    /// subscription filter. Defaults `5000`.
    #[serde(default = "defaults::relay::max_limit")]
    pub max_limit:        NonZeroUsize,
    /// Sets the default number of events returned when no limit is specified in
    /// a filter. Defaults `500`.
    #[serde(default = "defaults::relay::default_limit")]
    pub default_limit:    NonZeroUsize,
    /// The maximum allowed length for a subscription ID as a string. Defaults
    /// `150`.
    #[serde(default = "defaults::relay::max_subid_length")]
    pub max_subid_length: NonZeroUsize,
    /// List of allowed public keys in hex or `npub` format. Default is empty.
    #[serde(default, serialize_with = "parsers::pubkeys_ser")]
    pub whitelist:        ArcRwVec<PublicKey>,
    /// List of denied public keys in hex or `npub` format. Default is empty.
    #[serde(default, serialize_with = "parsers::pubkeys_ser")]
    pub blacklist:        ArcRwVec<PublicKey>,
    /// A list of administrators allowed to use the Relay Management API.
    #[serde(default, serialize_with = "parsers::pubkeys_ser")]
    pub admins:           ArcRwVec<PublicKey>,
    /// List of event kinds that are permitted
    #[serde(default, deserialize_with = "parsers::kind_de")]
    pub allowed_kinds:    ArcRwVec<Kind>,
    /// List of event kinds that are rejected
    #[serde(default, deserialize_with = "parsers::kind_de")]
    pub disallowed_kinds: ArcRwVec<Kind>,
}

/// Configuration for relay components.
#[derive(Debug, Deserialize, Serialize)]
pub struct RelayConfig {
    /// Network-related configs for the relay.
    #[serde(default)]
    pub net:       NetworkConfig,
    /// LMDB database configs for the relay.
    #[serde(default)]
    pub lmdb:      LmdbConfig,
    /// RateLimit configuration.
    #[serde(default)]
    pub ratelimit: RatelimitConfig,
    /// Relay's NIP-11 metadata
    #[serde(default)]
    pub nip11:     Nip11Config,
    /// Relay's plugins
    #[serde(default)]
    pub plugins:   PluginsConfig,
    #[serde(default)]
    pub grasp:     GraspConfig,
    /// Core relay-specific configs.
    #[serde(default, flatten)]
    pub relay:     CoreConfig,
}

impl RelayConfig {
    /// Reload the config from the env and config file
    pub fn reload() -> RelayResult<Self> {
        let config: RelayConfig = Config::builder()
            .add_source(
                FileSource::from(pathes::config_file_path())
                    .format(FileFormat::Toml)
                    .required(false),
            )
            .add_source(RelayConfig::from_env()?)
            .build()
            .unwrap()
            .try_deserialize()
            .map_err(|err| RelayError::Config(err.to_string()))?;

        Ok(config.post())
    }

    /// Post the config
    fn post(mut self) -> Self {
        self.nip11.limitation.max_message_length = self.relay.max_event_size.into();
        self.nip11.limitation.max_limit = self.relay.max_limit.into();
        self.nip11.limitation.default_limit = self.relay.default_limit.into();
        self.nip11.limitation.max_subid_length = self.relay.max_subid_length.into();
        self.nip11.limitation.max_subscriptions = self.relay.max_connections.unwrap_or(usize::MAX);
        self.nip11.limitation.min_pow_difficulty = self.relay.min_pow;
        self.nip11.limitation.auth_required = self.relay.nip42;
        self.nip11.limitation.restricted_writes = self.relay.min_pow > 0
            || self.relay.nip42
            || self.nip11.limitation.payment_required
            || !self.relay.whitelist.is_empty()
            || !self.relay.allowed_kinds.is_empty();

        self
    }

    /// Retrieves the database instance based on the configuration.
    pub async fn get_relay_db(&self) -> RelayResult<Arc<dyn nostr_database::NostrDatabase>> {
        // TODO: Support sqlite
        Ok(Arc::new(
            nostr_lmdb::NostrLmdbBuilder::new(&self.lmdb.dir)
                .map_size(self.lmdb.map_size)
                .max_readers(self.lmdb.max_readers)
                .additional_dbs(self.lmdb.additional_dbs)
                .build()
                .await?,
        ))
    }
}

impl Drop for RelayConfig {
    fn drop(&mut self) {
        fn inner(value: &RelayConfig) -> Result<(), Box<dyn std::error::Error>> {
            fs::write(pathes::config_file_path(), toml::to_string_pretty(value)?)?;
            Ok(())
        }

        tracing::info!("Writing the configuration to the file");

        SERIALIZE_TO_TOML.store(true, Ordering::Relaxed);
        if let Err(err) = inner(self) {
            tracing::error!("Failed to write configuration to file: {err}");
        }
        SERIALIZE_TO_TOML.store(false, Ordering::Relaxed);
    }
}

impl From<RatelimitConfig> for nostr_relay_builder::builder::RateLimit {
    fn from(value: RatelimitConfig) -> Self {
        Self {
            max_reqs:         value.max_queries,
            notes_per_minute: value.events_per_minute,
        }
    }
}

/// Returns whether serialization to TOML is enabled.
fn ser_for_toml<T>(_: T) -> bool {
    SERIALIZE_TO_TOML.load(Ordering::Relaxed)
}
