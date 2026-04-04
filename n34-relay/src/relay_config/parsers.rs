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

use std::{str::FromStr, sync::Arc};

use hyper::Uri;
use nostr::{event::Kind, key::PublicKey, nips::nip19::ToBech32};
use parking_lot::RwLock;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Serializes a vector of public keys into a list of bech32-encoded `npub`
/// strings.
pub fn pubkeys_ser<S: Serializer>(
    pubkeys: &Arc<RwLock<Vec<PublicKey>>>,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    pubkeys
        .read()
        .iter()
        .map(|pkey| pkey.to_bech32().expect("Infallible"))
        .collect::<Vec<_>>()
        .serialize(serializer)
}

/// Deserialize the kind from a number
pub fn kind_de<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<Arc<RwLock<Vec<Kind>>>, D::Error> {
    Ok(Arc::new(RwLock::new(
        Vec::<u16>::deserialize(deserializer)?
            .into_iter()
            .map(Kind::from)
            .collect(),
    )))
}

/// Serializes a vector of URIs into a list of strings.
pub fn uris_ser<S: Serializer>(uris: &[Uri], serializer: S) -> Result<S::Ok, S::Error> {
    uris.iter()
        .map(|uri| uri.to_string())
        .collect::<Vec<_>>()
        .serialize(serializer)
}

/// Deserialize a list of string into URIs
pub fn uris_de<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Vec<Uri>, D::Error> {
    Vec::<String>::deserialize(deserializer)?
        .into_iter()
        .map(|str_uri| Uri::from_str(&str_uri))
        .collect::<Result<Vec<Uri>, _>>()
        .map_err(|err| serde::de::Error::custom(err.to_string()))
}
