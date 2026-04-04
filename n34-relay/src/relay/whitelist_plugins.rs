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

use std::sync::Arc;

use nostr::{
    event::{Event, Kind, TagStandard, Tags},
    key::PublicKey,
    util::BoxedFuture,
};
use nostr_relay_builder::builder::WritePolicyResult;
use parking_lot::RwLock;

use crate::{
    ext_traits::{RwlockVecExt, WritePolicyResultExt},
    relay::plugins_manager::RelayPlugin,
};

type ArcRwVec<T> = Arc<RwLock<Vec<T>>>;

/// A blacklist of public keys. Events are accepted only if their public key is
/// not in this list.
pub struct PubKeyBlacklist(
    /// The set of blacklisted public keys.
    pub ArcRwVec<PublicKey>,
);

/// A whitelist of public keys. Events are accepted only if the whitelist is
/// empty or their public key is in it.
pub struct PubKeyWhiteList(
    /// The set of whitelisted public keys.
    pub ArcRwVec<PublicKey>,
);

/// Verifies if the event tags contain any of the whitelisted public keys.
/// Events with such mentions are accepted.
pub struct MentionedPubKey(
    /// The collection of public keys to search for in the event tags.
    pub ArcRwVec<PublicKey>,
);

/// Accepts events if their kind is in the whitelist or if the whitelist is
/// empty.
pub struct KindWhitelist(
    /// The list of allowed event kinds.
    pub ArcRwVec<Kind>,
);

/// Accepts events if their kind is not in the blacklist.
pub struct KindBlacklist(
    /// The list of disallowed event kinds.
    pub ArcRwVec<Kind>,
);

impl RelayPlugin for PubKeyBlacklist {
    fn check_event<'a>(&'a self, event: &'a Event) -> BoxedFuture<'a, Option<WritePolicyResult>> {
        Box::pin(async {
            if self.0.contains(&event.pubkey) {
                return Some(WritePolicyResult::blocked_reject(
                    "this public key is blacklisted",
                ));
            }
            Some(WritePolicyResult::Accept)
        })
    }
}

impl RelayPlugin for PubKeyWhiteList {
    fn check_event<'a>(&'a self, event: &'a Event) -> BoxedFuture<'a, Option<WritePolicyResult>> {
        Box::pin(async {
            if self.0.is_empty() || self.0.contains(&event.pubkey) {
                return Some(WritePolicyResult::Accept);
            }
            Some(WritePolicyResult::blocked_reject(
                "this public key is not whitelisted",
            ))
        })
    }
}

impl RelayPlugin for MentionedPubKey {
    fn check_event<'a>(&'a self, event: &'a Event) -> BoxedFuture<'a, Option<WritePolicyResult>> {
        Box::pin(async {
            let whitelist_lock = self.0.read();

            if whitelist_lock.is_empty()
                || public_keys(&event.tags).any(|p| whitelist_lock.contains(p))
            {
                return Some(WritePolicyResult::Accept);
            }

            Some(WritePolicyResult::blocked_reject(
                "this public key is not whitelisted",
            ))
        })
    }
}

impl RelayPlugin for KindWhitelist {
    fn check_event<'a>(&'a self, event: &'a Event) -> BoxedFuture<'a, Option<WritePolicyResult>> {
        Box::pin(async {
            if self.0.is_empty() || self.0.contains(&event.kind) {
                return Some(WritePolicyResult::Accept);
            }
            Some(WritePolicyResult::blocked_reject("not allowed event kind"))
        })
    }
}

impl RelayPlugin for KindBlacklist {
    fn check_event<'a>(&'a self, event: &'a Event) -> BoxedFuture<'a, Option<WritePolicyResult>> {
        Box::pin(async {
            if !self.0.contains(&event.kind) {
                return Some(WritePolicyResult::Accept);
            }
            Some(WritePolicyResult::blocked_reject("event kind is blocked"))
        })
    }
}

/// Returns all the public keys in the tags
#[inline]
fn public_keys(tags: &Tags) -> impl Iterator<Item = &PublicKey> {
    tags.iter().filter_map(|tag| {
        match tag.as_standardized()? {
            TagStandard::PublicKey { public_key, .. } => Some(public_key),
            TagStandard::PublicKeyReport(pkey, ..) => Some(pkey),
            TagStandard::Coordinate { coordinate, .. } => Some(&coordinate.public_key),
            _ => None,
        }
    })
}
