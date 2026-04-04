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

use nostr::{event::Kind, key::PublicKey, types::Url};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Public key paired with an optional reason.
#[derive(Serialize)]
pub struct PubKeyWithReason {
    #[serde(serialize_with = "hex_pubkey_ser")]
    pub pubkey: PublicKey,
    pub reason: Option<String>,
}

/// A kind
pub struct KindParam(pub Kind);

/// A public key
pub struct PublicKeyParam(pub PublicKey);

/// A single string
pub struct StringParam(pub String);

/// A signle URL
pub struct UrlParam(pub Url);

impl From<PublicKey> for PubKeyWithReason {
    fn from(pubkey: PublicKey) -> Self {
        Self {
            pubkey,
            reason: None,
        }
    }
}

impl<'de> Deserialize<'de> for PubKeyWithReason {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let params: Vec<String> = Vec::deserialize(deserializer)?;

        let pubkey_str = params
            .first()
            .ok_or_else(|| de_err::<D>("Missing required parameter: public key"))?;
        let pubkey = PublicKey::parse(pubkey_str)
            .map_err(|_| de_err::<D>(format!("Invalid public key format: '{pubkey_str}'")))?;
        let reason = params.into_iter().nth(1);

        Ok(Self { pubkey, reason })
    }
}

impl<'de> Deserialize<'de> for KindParam {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(Self(Kind::from(
            *Vec::<u16>::deserialize(deserializer)?
                .first()
                .ok_or_else(|| de_err::<D>("Missing required parameter: kind"))?,
        )))
    }
}

impl<'de> Deserialize<'de> for StringParam {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(Self(
            Vec::<String>::deserialize(deserializer)?
                .into_iter()
                .next()
                .ok_or_else(|| {
                    de_err::<D>("Expected at least one string in the list, but found none")
                })?,
        ))
    }
}

impl<'de> Deserialize<'de> for PublicKeyParam {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(Self(
            PublicKey::parse(&StringParam::deserialize(deserializer)?.0)
                .map_err(|_| de_err::<D>("Invalid public key format"))?,
        ))
    }
}

impl<'de> Deserialize<'de> for UrlParam {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(Self(
            Url::parse(&StringParam::deserialize(deserializer)?.0)
                .map_err(|_| de_err::<D>("Invalid URL"))?,
        ))
    }
}

/// Converts the public key to a hex string for serialization.
///
/// Note: While this functionality is also provided by the `Serialize`
/// implementation for `PublicKey`, it is included here to guard against
/// potential breaking changes in the beta crate.
pub fn hex_pubkey_ser<S: Serializer>(pubkey: &PublicKey, serializer: S) -> Result<S::Ok, S::Error> {
    pubkey.to_hex().serialize(serializer)
}


/// A function to return a custom deserializing error
fn de_err<'de, D: Deserializer<'de>>(err: impl AsRef<str>) -> D::Error {
    serde::de::Error::custom(err.as_ref())
}
