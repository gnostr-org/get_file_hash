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
    Extension,
    body::Bytes,
    response::{IntoResponse, Response},
};
use hyper::StatusCode;
use nostr::nips::nip19::ToBech32;

use super::{git_command::GitCommand, ref_update_pkt_line::parse_pkt_lines, utils};
use crate::router_state::RouterState;

/// Handles a git-receive-pack request for a repository.
/// Verifies the repository exists and processes the received pack data.
/// Returns a successful response with the pack data or an appropriate error.
pub async fn receive_pack(
    Extension(state): Extension<Arc<RouterState>>,
    params: super::PublicKeyAndRepoPath,
    body: Bytes,
) -> Response {
    let Some(repo_path) = state.repo_path(&params.public_key, &params.repo_name) else {
        return (StatusCode::NOT_FOUND, "Repository not found").into_response();
    };

    let ref_updates = match parse_pkt_lines(&body) {
        Ok(ref_updates) => ref_updates,
        Err(err) => return (StatusCode::BAD_REQUEST, err).into_response(),
    };

    tracing::debug!(
        "Ref updates: {ref_updates:?} of repo `{}/{}.git`",
        params.public_key.to_bech32().expect("Infallible"),
        params.repo_name
    );

    let capabilities = ref_updates
        .iter()
        .find_map(|ref_update| ref_update.capabilities)
        .unwrap_or_default();

    let refs_errors = match utils::is_legal_push(&ref_updates, &state.database, &params).await {
        Ok(refs) => refs,
        Err((status_code, err)) => {
            tracing::error!(err = %err, "Failed to check the ref updates");
            return (status_code, err).into_response();
        }
    };

    if !refs_errors.is_empty() {
        return utils::git_receive_pack_error(&refs_errors, &ref_updates, capabilities);
    }

    match GitCommand::new(&state.config.grasp.git_path, &repo_path)
        .receive_pack(&body)
        .await
    {
        Ok(response_body) => {
            Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", "application/x-git-receive-pack-result")
                .header("Connection", "Keep-Alive")
                .header("Transfer-Encoding", "chunked")
                .header("X-Content-Type-Options", "nosniff")
                .body(response_body)
                .expect("valid response")
        }
        Err(err_msg) => (StatusCode::INTERNAL_SERVER_ERROR, err_msg).into_response(),
    }
}
