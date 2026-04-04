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
    body::Body,
    http::HeaderValue,
    response::{IntoResponse, Response},
};
use hyper::{StatusCode, header};

use crate::{git_server::GitFilePath, router_state::RouterState};

/// Fetches the content of a file in a repository using the provided
/// `GitFilePath`. The function checks if the repository and file exist, and
/// handles caching headers based on the `CACHE_HEADER` flag.
pub async fn get_file_content<const CACHE_HEADER: bool>(
    Extension(state): Extension<Arc<RouterState>>,
    params: super::PublicKeyAndRepoPath,
    GitFilePath(file_path): GitFilePath,
) -> Response {
    let Some(repo_path) = state.repo_path(&params.public_key, &params.repo_name) else {
        return (StatusCode::NOT_FOUND, "Repository not found").into_response();
    };

    let file_path = repo_path.join(file_path);

    if !file_path.exists() {
        return (StatusCode::NOT_FOUND, "File not found").into_response();
    }

    let mut response = Response::builder();
    let headers = response
        .headers_mut()
        .expect("builder function provide the response parts");

    if CACHE_HEADER {
        let expires_value = (chrono::Utc::now() + chrono::Duration::days(1)).to_rfc2822();
        headers.insert(
            "Cache-Control",
            HeaderValue::from_static("public, max-age=86400"),
        );
        headers.insert(
            "Expires",
            HeaderValue::from_str(&expires_value).expect("valid header value"),
        );

        let content_type = if file_path.extension().is_some_and(|ext| ext == "pack") {
            "application/x-git-packed-objects"
        } else if file_path.extension().is_some_and(|ext| ext == "idx") {
            "application/x-git-packed-objects-toc"
        } else {
            "application/x-git-loose-object"
        };

        headers.insert(header::CONTENT_TYPE, HeaderValue::from_static(content_type));
    } else {
        headers.insert("Pragma", HeaderValue::from_static("no-cache"));
        headers.insert("Cache-Control", crate::git_server::CACHE_CONTROL_NO_CACHE);
        headers.insert("Expires", crate::git_server::EXPIRES_NO_CACHE);
        headers.insert(header::CONTENT_TYPE, HeaderValue::from_static("text/plain"));
    }

    let Ok(file_content) = tokio::fs::read(&file_path).await else {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to get file content",
        )
            .into_response();
    };

    Response::builder()
        .status(StatusCode::OK)
        .body(Body::from(file_content))
        .expect("valid response")
}
