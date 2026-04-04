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

use std::path::{Component, PathBuf};

use axum::{extract::FromRequestParts, http::request::Parts};
use hyper::StatusCode;
use nostr::key::PublicKey;

/// Parameters containing a repository author's public key and repository name.
#[derive(serde::Deserialize)]
pub struct PublicKeyAndRepoPath {
    /// The author's Nostr public key.
    pub public_key: PublicKey,
    /// The full repository name.
    pub repo_name:  String,
}

/// Either `git-upload-pack` or `git-receive-pack`
#[derive(serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ServiceName {
    GitUploadPack,
    GitReceivePack,
}

/// Query parameter containing the service name (`?service=<string>`)
#[derive(serde::Deserialize)]
pub struct ServiceQuery {
    /// The service name
    pub service: ServiceName,
}

/// An extractor that indecates what file to return its content, it's used with
/// `get_file_content` endpoint only.
pub struct GitFilePath(pub PathBuf);

impl ServiceName {
    /// Gets the service name without the `git-` prefix
    pub const fn name(&self) -> &'static str {
        match self {
            Self::GitUploadPack => "upload-pack",
            Self::GitReceivePack => "receive-pack",
        }
    }

    /// Gets the pkt-line header of the service
    pub const fn pkt_line_header(&self) -> &[u8] {
        match self {
            Self::GitUploadPack => "001e# service=git-upload-pack\n0000".as_bytes(),
            Self::GitReceivePack => "001f# service=git-receive-pack\n0000".as_bytes(),
        }
    }
}

impl<S> FromRequestParts<S> for GitFilePath
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        // Skip `/<npub>/repo.git/`
        let path = PathBuf::from(
            parts
                .uri
                .path()
                .trim_matches('/')
                .split('/')
                .skip(2)
                .collect::<Vec<_>>()
                .join("/"),
        );

        if path.components().any(|component| {
            matches!(
                component,
                Component::CurDir | Component::ParentDir | Component::Prefix(..)
            )
        }) {
            return Err((StatusCode::BAD_REQUEST, "Invalid path"));
        };

        if path.as_os_str() == "HEAD"
            || path.starts_with("objects/info/")
            || path.starts_with("objects/pack/")
        {
            Ok(Self(path.to_path_buf()))
        } else {
            Err((StatusCode::BAD_REQUEST, "Unknown path"))
        }
    }
}

impl<S> FromRequestParts<S> for PublicKeyAndRepoPath
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, String);

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        use axum::extract::Path;

        let Path(mut pkey_and_repo) =
            Path::<PublicKeyAndRepoPath>::from_request_parts(parts, state)
                .await
                .map_err(|err| (StatusCode::BAD_REQUEST, err.to_string()))?;

        if pkey_and_repo.repo_name.ends_with(".git") {
            pkey_and_repo.repo_name = pkey_and_repo.repo_name.trim_end_matches(".git").to_owned();
            return Ok(pkey_and_repo);
        }

        Err((
            StatusCode::BAD_REQUEST,
            "The repository name must ends with `.git`".to_owned(),
        ))
    }
}
