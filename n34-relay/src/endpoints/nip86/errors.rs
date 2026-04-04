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

use axum::response::{IntoResponse, Response};
use hyper::StatusCode;
use serde::Serialize;

use crate::endpoints::nip86::responses::Nip86Response;

/// API result type
pub type ApiResult<T> = Result<T, ApiError>;

/// API errors
#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("Missing 'Authorization' header in request")]
    NoAuthHeader,
    #[error("Invalid authorization header: contains non-ASCII characters")]
    InvalidAuthHeader,
    #[error("Authorization header must start with 'Nostr ' prefix")]
    NotNostrAuth,
    #[error("Invalid Base64 encoding in authorization token")]
    InvalidBase64Token,
    #[error("Invalid Nostr event in authorization token")]
    InvalidNostrEventToken,
    #[error("Access denied: administrator privileges required")]
    NotAdmin,
    #[error("Access denied: this request is restricted to super administrator only")]
    NotSuperAdmin,
    #[error("Invalid event ID in authorization token")]
    InvalidEventId,
    #[error("Authorization token must use event kind 27235")]
    NotHttpAuthKind,
    #[error("Authorization token event content must be empty")]
    NonEmptyHttpAuthContent,
    #[error("Authorization token has expired")]
    OldAuthEvent,
    #[error("Missing the relay url in authorization token")]
    MissingRlayUrl,
    #[error("Token relay URL does not match current relay URL")]
    IncorrectRelayUrl,
    #[error("Missing the request method in authorization token")]
    MissingMethod,
    #[error("Token request method does not match actual request method")]
    IncorrectRequestMethod,
    #[error("Invalid event signature in authorization token")]
    InvalidSignature,
    #[error("Missing payload hash in authorization token")]
    MissingPayloadHash,
    #[error("Invalid request body: {0}")]
    InvalidBody(String),
    #[error("Body hash does not match payload tag in authorization token")]
    IncorrectBodyHash,
    #[error("Invalid RPC request: {0}")]
    InvalidRpcRequest(String),
}

impl ApiError {
    /// Retrieves the HTTP status code associated with the error
    #[inline]
    pub fn status_code(&self) -> StatusCode {
        match self {
            Self::InvalidRpcRequest(_) => StatusCode::BAD_REQUEST,
            _ => StatusCode::UNAUTHORIZED,
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        Nip86Response::err_res(self).into_response()
    }
}

impl Serialize for ApiError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.to_string().serialize(serializer)
    }
}
