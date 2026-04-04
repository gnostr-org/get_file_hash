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

use axum::{
    body::Bytes,
    extract::{FromRequest, Request},
    response::{IntoResponse, Response},
};
use nostr::hashes::hex::DisplayHex;
use sha1::Digest;

use crate::{
    endpoints::nip86::{
        errors::{ApiError, ApiResult},
        requests::Nip86Request,
    },
    ext_traits::RwlockVecExt,
    router_state::RouterState,
};

/// API errors
mod errors;
/// Requests parameters
mod params;
/// API requests
mod requests;
/// API responses
mod responses;
/// API utils
mod utils;

/// A main NIP86 API handler.
pub async fn main_nip86_handler(state: Arc<RouterState>, request: Request) -> ApiResult<Response> {
    let (request_author, payload_hash) = utils::get_payload_hash(
        utils::get_auth_token(request.headers())?,
        request.method(),
        state.as_ref(),
    )?;

    let request_body = Bytes::from_request(request, &())
        .await
        .map_err(|err| ApiError::InvalidBody(err.to_string()))?;

    if payload_hash != sha2::Sha256::digest(&request_body).as_hex().to_string() {
        return Err(ApiError::IncorrectBodyHash);
    }

    let rpc_request: Nip86Request = serde_json::from_slice(&request_body).map_err(|err| {
        ApiError::InvalidRpcRequest(utils::remove_line_number(&err.to_string()).to_owned())
    })?;

    if rpc_request.only_superadmin() && !state.config.relay.admins.is_first(&request_author) {
        return Err(ApiError::NotSuperAdmin);
    }

    rpc_request
        .run(state)
        .await
        .map(IntoResponse::into_response)
}
