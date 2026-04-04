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

use axum::{
    Json,
    response::{IntoResponse, Response},
};
use hyper::StatusCode;
use nostr::{event::Kind, key::PublicKey};
use serde::{Serialize, Serializer};

use crate::endpoints::nip86::{errors::ApiError, params::PubKeyWithReason};

/// API response body containing result and error information
#[derive(Serialize)]
pub struct Nip86Response {
    /// Operation result
    #[serde(skip_serializing_if = "Nip86Result::is_empty")]
    result: Nip86Result,
    /// Error message if operation failed
    #[serde(skip_serializing_if = "Option::is_none")]
    error:  Option<ApiError>,
}

/// Possible results for a NIP86 API response
#[derive(Serialize)]
#[serde(untagged)]
pub enum Nip86Result {
    Empty,
    #[serde(serialize_with = "always_true")]
    True,
    PublicKeysAndReason(Vec<PubKeyWithReason>),
    Kinds(Vec<Kind>),
    #[serde(serialize_with = "hex_pubkeys_ser")]
    PublicKeys(Vec<PublicKey>),
    SupportedMethods(&'static [&'static str]),
}

impl Nip86Result {
    /// Checks if the result is empty.
    pub fn is_empty(&self) -> bool {
        matches!(self, Nip86Result::Empty)
    }
}

impl Nip86Response {
    /// Creates a new successful response with the given result.
    pub fn ok_res(result: Nip86Result) -> Self {
        Self {
            result,
            error: None,
        }
    }

    /// Creates a new error response with the provided error.
    pub fn err_res(err: ApiError) -> Self {
        Self {
            result: Nip86Result::Empty,
            error:  Some(err),
        }
    }
}

impl IntoResponse for Nip86Response {
    fn into_response(self) -> Response {
        (
            self.error
                .as_ref()
                .map(|err| err.status_code())
                .unwrap_or(StatusCode::OK),
            Json(self),
        )
            .into_response()
    }
}

impl IntoResponse for Nip86Result {
    fn into_response(self) -> Response {
        Nip86Response::ok_res(self).into_response()
    }
}

/// function that returns true bool in serializeation
fn always_true<S>(s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    true.serialize(s)
}


/// Serialize to a list of hex public keys
fn hex_pubkeys_ser<S>(pkeys: &[PublicKey], s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    pkeys
        .iter()
        .map(PublicKey::to_hex)
        .collect::<Vec<_>>()
        .serialize(s)
}
