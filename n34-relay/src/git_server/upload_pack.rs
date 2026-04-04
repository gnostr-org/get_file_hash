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

use super::git_command::GitCommand;
use crate::router_state::RouterState;

/// Handles a git-upload-pack request for a repository.
/// Verifies the repository exists and processes the received pack data.
/// Returns a successful response with the pack data or an appropriate error.
pub async fn upload_pack(
    Extension(state): Extension<Arc<RouterState>>,
    params: super::PublicKeyAndRepoPath,
    headers: hyper::HeaderMap,
    body: Bytes,
) -> Response {
    let Some(repo_path) = state.repo_path(&params.public_key, &params.repo_name) else {
        return (StatusCode::NOT_FOUND, "Repository not found").into_response();
    };

    match GitCommand::new(&state.config.grasp.git_path, &repo_path)
        .upload_pack(&body, super::utils::contains_git_v2(&headers))
        .await
    {
        Ok(response_body) => {
            Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", "application/x-git-upload-pack-result")
                .header("Connection", "Keep-Alive")
                .header("Transfer-Encoding", "chunked")
                .header("X-Content-Type-Options", "nosniff")
                .body(response_body)
                .expect("valid response")
        }
        Err(err_msg) => (StatusCode::INTERNAL_SERVER_ERROR, err_msg).into_response(),
    }
}
