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

use std::time::{SystemTime, UNIX_EPOCH};

use axum::http::HeaderValue;
use base64::{
    Engine,
    prelude::{BASE64_STANDARD, BASE64_STANDARD_NO_PAD, BASE64_URL_SAFE, BASE64_URL_SAFE_NO_PAD},
};
use hyper::{HeaderMap, Method};
use nostr::{
    event::{Event, Kind, Tag, TagKind},
    filter::Alphabet,
    key::PublicKey,
    util::JsonUtil,
};

use crate::{
    endpoints::nip86::errors::{ApiError, ApiResult},
    ext_traits::RwlockVecExt,
    router_state::RouterState,
    utils as crate_utils,
};

const NOSTR_SCHEME: &str = "Nostr ";

/// Extracts the authentication token from the headers, ensuring it matches the
/// expected scheme.
pub fn get_auth_token(headers: &HeaderMap<HeaderValue>) -> ApiResult<&str> {
    headers
        .get("authorization")
        .ok_or(ApiError::NoAuthHeader)
        .and_then(|header| header.to_str().map_err(|_| ApiError::InvalidAuthHeader))
        .and_then(|auth_token| {
            if let Some(("", auth_token)) = auth_token.trim().split_once(NOSTR_SCHEME) {
                Ok(auth_token)
            } else {
                Err(ApiError::NotNostrAuth)
            }
        })
}

/// Verifies a NIP-98 HTTP auth token and returns its author and payload hash if
/// valid.
///
/// The token must be from an admin, properly signed, and match the request
/// method and relay URL.
#[must_use = "payload hash must be checked"]
pub fn get_payload_hash(
    base64_str: impl AsRef<str>,
    request_method: &Method,
    state: &RouterState,
) -> ApiResult<(PublicKey, String)> {
    // Attempt to decode the base64 string using various base64 standards (standard,
    // standard without padding, URL-safe, and URL-safe without padding) since
    // NIP-98 (HTTP Auth) does not specify which base64 variant to use.
    let event = Event::from_json(
        BASE64_STANDARD
            .decode(base64_str.as_ref())
            .or_else(|_| BASE64_STANDARD_NO_PAD.decode(base64_str.as_ref()))
            .or_else(|_| BASE64_URL_SAFE.decode(base64_str.as_ref()))
            .or_else(|_| BASE64_URL_SAFE_NO_PAD.decode(base64_str.as_ref()))
            .map_err(|_| ApiError::InvalidBase64Token)?,
    )
    .map_err(|_| ApiError::InvalidNostrEventToken)?;

    if !state.config.relay.admins.contains(&event.pubkey) {
        return Err(ApiError::NotAdmin);
    }

    if !event.verify_id() {
        return Err(ApiError::InvalidEventId);
    }

    if event.kind != Kind::HttpAuth {
        return Err(ApiError::NotHttpAuthKind);
    }

    if !event.content.is_empty() {
        return Err(ApiError::NonEmptyHttpAuthContent);
    }

    if let Ok(current_time) = SystemTime::now().duration_since(UNIX_EPOCH)
        && (current_time.as_secs() - 100) > event.created_at.as_secs()
    {
        return Err(ApiError::OldAuthEvent);
    }

    if crate_utils::remove_proto(
        event
            .tags
            .find(TagKind::single_letter(Alphabet::U, false))
            .and_then(Tag::content)
            .ok_or(ApiError::MissingRlayUrl)?,
    ) != state.config.relay.domain.as_str()
    {
        return Err(ApiError::IncorrectRelayUrl);
    }

    if event
        .tags
        .find(TagKind::Method)
        .and_then(Tag::content)
        .ok_or(ApiError::MissingMethod)?
        != request_method.as_str()
    {
        return Err(ApiError::IncorrectRequestMethod);
    }

    if !event.verify_signature() {
        return Err(ApiError::InvalidSignature);
    }

    Ok((
        event.pubkey,
        event
            .tags
            .find(TagKind::Payload)
            .and_then(Tag::content)
            .ok_or(ApiError::MissingPayloadHash)?
            .to_owned(),
    ))
}

/// Removes the line number and column details from a serde_json error message,
/// if present.
pub fn remove_line_number(err: &str) -> &str {
    err.rsplit_once(" at line ").map_or(err, |(msg, _)| msg)
}
