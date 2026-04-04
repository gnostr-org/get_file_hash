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

use std::{borrow::Cow, sync::Arc};

use axum::http::HeaderValue;
use hyper::{
    HeaderMap,
    header::{self, AsHeaderName},
};
use nostr::message::MachineReadablePrefix;
use nostr_relay_builder::builder::WritePolicyResult;
use parking_lot::RwLock;

/// Extension trait for managing a shared list of things
#[easy_ext::ext(RwlockVecExt)]
pub impl<T: PartialEq> Arc<RwLock<Vec<T>>> {
    /// Construct a new instance
    fn new_empty() -> Self {
        Arc::new(RwLock::new(Vec::new()))
    }

    /// Check if the list is empty
    fn is_empty(&self) -> bool {
        self.read().is_empty()
    }

    /// Checks if the given value exists in the list
    fn contains(&self, val: &T) -> bool {
        self.read().contains(val)
    }

    /// Returns `true` if the giveing value is the first value in the list
    fn is_first(&self, value: &T) -> bool {
        self.read()
            .first()
            .is_some_and(|first_value| first_value == value)
    }
}

/// Extension trait for `RwLock` options
#[easy_ext::ext(RwlockOption)]
pub impl<T> RwLock<Option<T>> {
    /// Returns true if the option is none
    fn is_none(&self) -> bool {
        self.read().is_none()
    }
}

/// Extension trait for header map
#[easy_ext::ext(HeaderMapExt)]
pub impl &HeaderMap<HeaderValue> {
    const NOSTR_JSON_MIME: &'static str = "application/nostr+json";
    const NOSTR_JSON_RPC_MIME: &'static str = "application/nostr+json+rpc";
    const UPGRADE_MIME: &'static str = "upgrade";
    const WEBSOCKET_MIME: &'static str = "websocket";

    /// Checks if the header map contains the specified header and if its value
    /// matches the given value.
    #[inline]
    fn is_contains(&self, header_name: impl AsHeaderName, header_value: &str) -> bool {
        self.get(header_name)
            .and_then(|content_value| content_value.to_str().ok())
            .is_some_and(|content_str| content_str.eq_ignore_ascii_case(header_value))
    }

    /// Checks if the provided headers indicate an upgrade to a WebSocket
    /// connection.
    fn is_ws_upgrade(&self) -> bool {
        self.is_contains(header::CONNECTION, Self::UPGRADE_MIME)
            && self.is_contains(header::UPGRADE, Self::WEBSOCKET_MIME)
    }

    /// Checks if the provided headers indicate a NIP-86 request
    fn is_nip86_req(&self) -> bool {
        self.is_contains(header::CONTENT_TYPE, Self::NOSTR_JSON_RPC_MIME)
    }

    /// Checks if the provided headers indicate a NIP-11 request
    fn is_nip11_req(&self) -> bool {
        self.is_contains(header::ACCEPT, Self::NOSTR_JSON_MIME)
    }
}

/// Extension trait for [WritePolicyResult]
#[easy_ext::ext(WritePolicyResultExt)]
pub impl WritePolicyResult {
    /// Reject result with `Blocked` prefix
    #[inline]
    fn blocked_reject<S>(msg: S) -> Self
    where
        S: Into<Cow<'static, str>>,
    {
        WritePolicyResult::reject(MachineReadablePrefix::Blocked, msg)
    }
}
