// n34 - A CLI to interact with NIP-34 and other stuff related to codes in nostr
// Copyright (C) 2025 Awiteb <a@4rs.nl>
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://gnu.org/licenses/gpl-3.0.html>.

use std::{collections::HashSet, fs, net::SocketAddr, path::PathBuf};

use nostr::{
    nips::{nip19::Nip19Coordinate, nip46::NostrConnectURI},
    types::RelayUrl,
};

use crate::{
    cli::traits::{MutRepoRelaySetsExt, RepoRelaySetsExt},
    error::N34Result,
};

/// Errors that can occur when working with configuration files.
#[derive(thiserror::Error, Debug)]
pub enum ConfigError {
    #[error(
        "Could not determine the default config path: both `$XDG_CONFIG_HOME` and `$HOME` \
         environment variables are missing or unset."
    )]
    CanNotFindConfigPath,
    #[error("Couldn't read the config file: {0}")]
    ReadFile(std::io::Error),
    #[error("Couldn't write in the config file: {0}")]
    WriteFile(std::io::Error),
    #[error("Couldn't serialize the config. This is a bug, please report it: {0}")]
    Serialize(toml::ser::Error),
    #[error("Failed to parse the config file: {0}")]
    ParseFile(toml::de::Error),
    #[error("Duplicate configuration set name detected: '{0}'. Each set  must have a unique name.")]
    SetDuplicateName(String),
    #[error("No set with the given name `{0}`")]
    SetNotFound(String),
    #[error("You can't create an new empty set.")]
    NewEmptySet,
}

/// Configuration for the command-line interface.
#[derive(serde::Serialize, serde::Deserialize, Clone, Default, Debug)]
pub struct CliConfig {
    /// Path to the configuration file (not serialized)
    #[serde(skip)]
    path:                   PathBuf,
    /// Groups of repositories and relays.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub sets:               Vec<RepoRelaySet>,
    /// The default PoW difficulty
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pow:                Option<u8>,
    /// List of fallback relays used if no fallback relays was provided.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fallback_relays:    Option<Vec<RelayUrl>>,
    /// Default Nostr bunker URL used for signing events.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "super::parsers::de_bunker_url",
        serialize_with = "super::parsers::ser_bunker_url"
    )]
    pub bunker_url:         Option<NostrConnectURI>,
    /// Whether to use the system keyring to store the secret key.
    #[serde(default)]
    pub keyring_secret_key: bool,
    /// Signs events using the browser's NIP-07 extension.
    #[serde(default)]
    pub nip07:              Option<SocketAddr>,
}

/// A named group of repositories and relays.
#[derive(serde::Serialize, serde::Deserialize, Default, Clone, Debug)]
pub struct RepoRelaySet {
    /// Unique identifier for this group.
    pub name:   String,
    /// Repository addresses in this group.
    #[serde(
        default,
        skip_serializing_if = "HashSet::is_empty",
        serialize_with = "super::parsers::ser_naddrs",
        deserialize_with = "super::parsers::de_naddrs"
    )]
    pub naddrs: HashSet<Nip19Coordinate>,
    /// Relay URLs in this group.
    #[serde(default, skip_serializing_if = "HashSet::is_empty")]
    pub relays: HashSet<RelayUrl>,
}

impl CliConfig {
    /// Reads and parse a TOML config file from the given path, creating it if
    /// missing.
    pub fn load(file_path: PathBuf) -> N34Result<Self> {
        tracing::info!(path = %file_path.display(), "Loading configuration from file");
        // Make sure the file is exist
        if let Some(parent) = file_path.parent()
            && !parent.exists()
        {
            fs::create_dir_all(parent)?;
        }

        let _ = fs::File::create_new(&file_path);

        let mut config: Self =
            toml::from_str(&fs::read_to_string(&file_path).map_err(ConfigError::ReadFile)?)
                .map_err(ConfigError::ParseFile)?;
        config.path = file_path;

        config.post_sets()?;

        Ok(config)
    }

    /// Dump the config as toml in a file
    pub fn dump(mut self) -> N34Result<()> {
        tracing::debug!(config = ?self, "Writing configuration to {}", self.path.display());
        self.post_sets()?;

        fs::write(
            &self.path,
            toml::to_string_pretty(&self).map_err(ConfigError::Serialize)?,
        )
        .map_err(ConfigError::WriteFile)?;

        Ok(())
    }

    /// Performs post-processing validation on the sets after loading or before
    /// dumping.
    fn post_sets(&mut self) -> N34Result<()> {
        self.sets.as_slice().ensure_names()?;
        self.sets.dedup_naddrs();

        Ok(())
    }
}

impl RepoRelaySet {
    /// Create a new [`RepoRelaySet`]
    pub fn new(
        name: impl Into<String>,
        naddrs: impl IntoIterator<Item = Nip19Coordinate>,
        relays: impl IntoIterator<Item = RelayUrl>,
    ) -> Self {
        Self {
            name:   name.into(),
            naddrs: HashSet::from_iter(naddrs),
            relays: HashSet::from_iter(relays),
        }
    }

    /// Removes duplicate repository addresses by comparing their coordinates,
    /// ignoring embedded relays.
    pub fn dedup_naddrs(&mut self) {
        let mut seen = HashSet::new();
        self.naddrs.retain(|n| seen.insert(n.coordinate.clone()));
    }
}
