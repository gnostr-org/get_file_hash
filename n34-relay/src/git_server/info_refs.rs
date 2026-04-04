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
    extract::Query,
    response::{IntoResponse, Response},
};
use hyper::{HeaderMap, StatusCode};

use super::{ServiceQuery, git_command::GitCommand};
use crate::router_state::RouterState;

/// Retrieves Git repository reference information for a given service.
/// Returns a response containing the references or an appropriate error message
/// if the repository or service is not found.
pub async fn info_refs(
    Extension(state): Extension<Arc<RouterState>>,
    params: super::PublicKeyAndRepoPath,
    Query(ServiceQuery { service }): Query<ServiceQuery>,
    headers: HeaderMap,
) -> Response {
    let Some(repo_path) = state.repo_path(&params.public_key, &params.repo_name) else {
        return (StatusCode::NOT_FOUND, "Repository not found").into_response();
    };

    match GitCommand::new(&state.config.grasp.git_path, &repo_path)
        .refs(&service, super::utils::contains_git_v2(&headers))
        .await
    {
        Ok(response_body) => {
            Response::builder()
                .status(StatusCode::OK)
                .header(
                    "Content-Type",
                    format!("application/x-git-{}-advertisement", service.name()),
                )
                .header("Pragma", "no-cache")
                .header("Cache-Control", crate::git_server::CACHE_CONTROL_NO_CACHE)
                .header("Expires", crate::git_server::EXPIRES_NO_CACHE)
                .body(response_body)
                .expect("valid response")
        }
        Err(err_msg) => (StatusCode::INTERNAL_SERVER_ERROR, err_msg).into_response(),
    }
}
