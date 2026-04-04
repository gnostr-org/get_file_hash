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

use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
};

use nostr::{
    Kind,
    nips::{
        nip19::{FromBech32, Nip19Coordinate, ToBech32},
        nip46::NostrConnectURI,
    },
};
use serde::{Deserialize, Serialize, Serializer};

use super::CliConfig;
use crate::{
    cli::DEFAULT_FALLBACK_PATH,
    error::{N34Error, N34Result},
};

pub fn parse_repo_naddr(repo_naddr: &str) -> Result<Nip19Coordinate, String> {
    let naddr = Nip19Coordinate::from_bech32(repo_naddr).map_err(|err| err.to_string())?;
    if naddr.relays.is_empty() {
        tracing::warn!("The repository naddr does not contain any relay hints");
    }

    (naddr.kind == Kind::GitRepoAnnouncement)
        .then_some(naddr)
        .ok_or_else(|| "Invalid naddr: must be of kind 30617 (GitRepoAnnouncement)".to_owned())
}

/// Parses a nostr-address file into a NIP-19 coordinates. Expects the file to
/// contain a repository announcements.
pub fn parse_nostr_address_file(file_path: &Path) -> N34Result<Vec<Nip19Coordinate>> {
    let addresses = fs::read_to_string(file_path)
        .map_err(N34Error::CanNotReadNostrAddressFile)?
        .split("\n")
        .filter_map(|line| {
            (!line.starts_with("#") && !line.trim().is_empty())
                .then_some(parse_repo_naddr(line).map_err(N34Error::InvalidNostrAddressFileContent))
        })
        .collect::<N34Result<Vec<Nip19Coordinate>>>()?;
    if addresses.is_empty() {
        return Err(N34Error::EmptyNostrAddressFile);
    }
    Ok(addresses)
}

/// Loads CLI configuration from given path. Uses default config path if input
/// matches fallback path.
pub fn parse_config_path(config_path: &str) -> N34Result<CliConfig> {
    let mut path = PathBuf::from(config_path.trim());

    if config_path == DEFAULT_FALLBACK_PATH {
        path = super::defaults::config_path()?;
    };

    CliConfig::load(path)
}

/// Parses a bunker URL and checks if it's a valid Nostr Connect URI.
/// Returns an error if the URL is not a valid bunker URL.
pub fn parse_bunker_url(bunker_url: &str) -> N34Result<NostrConnectURI> {
    match NostrConnectURI::parse(bunker_url) {
        Ok(url) if url.is_bunker() => Ok(url),
        _ => Err(N34Error::NotBunkerUrl),
    }
}

/// Serializes a set of NIP-19 coordinates as a list of bech32 strings.
pub fn ser_naddrs<S>(naddr: &HashSet<Nip19Coordinate>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let str_naddrs = naddr
        .iter()
        .map(|n| n.to_bech32().map_err(|err| err.to_string()))
        .collect::<Result<Vec<_>, _>>()
        .map_err(serde::ser::Error::custom)?;

    str_naddrs.serialize(serializer)
}

/// Deserializes a list of bech32 strings into a set of NIP-19 coordinates.
pub fn de_naddrs<'de, D>(deserializer: D) -> Result<HashSet<Nip19Coordinate>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    Vec::<String>::deserialize(deserializer)?
        .into_iter()
        .map(|naddr| Nip19Coordinate::from_bech32(&naddr))
        .collect::<Result<HashSet<_>, _>>()
        .map_err(serde::de::Error::custom)
}

pub fn ser_bunker_url<S>(
    bunker_url: &Option<NostrConnectURI>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    bunker_url
        .as_ref()
        .map(|u| u.to_string())
        .serialize(serializer)
}

pub fn de_bunker_url<'de, D>(deserializer: D) -> Result<Option<NostrConnectURI>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    Option::<String>::deserialize(deserializer)?
        .map(|u| parse_bunker_url(&u))
        .transpose()
        .map_err(serde::de::Error::custom)
}
