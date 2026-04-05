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

use std::time::Duration;

use axum::{
    Router,
    http::HeaderValue,
    routing::{get, post},
};
use tower::limit::ConcurrencyLimitLayer;
use tower_http::timeout::TimeoutLayer;

use crate::relay_config::RelayConfig;

/// Git endpoint to return a file content
mod get_file_content;
/// Git command
pub mod git_command;
/// Git `info/refs` endpoint
mod info_refs;
/// Path and query params
mod path_and_query;
/// Git receive pack endpoint
mod receive_pack;
/// Parses the ref update packet lines
pub mod ref_update_pkt_line;
/// Git upload pack endpoint
mod upload_pack;
/// Git utils
pub mod utils;

use get_file_content::get_file_content;
use info_refs::info_refs;
pub use path_and_query::*;
use receive_pack::receive_pack;
use upload_pack::upload_pack;


/// The `Expires` header value to prevent caching (sets date far in the past)
const EXPIRES_NO_CACHE: HeaderValue = HeaderValue::from_static("Fri, 01 Jan 1980 00:00:00 GMT");
/// The `Cache-Control` header value to prevent caching
const CACHE_CONTROL_NO_CACHE: HeaderValue =
    HeaderValue::from_static("no-cache, max-age=0, must-revalidate");

/// Creates a router with paths prefixed by `/{public_key}/{repo_name}`.
///
/// # Example
/// ```rust
/// use axum::routing::get;
/// use axum::Router;
/// use n34_relay::git_router;
///
/// # fn main() {
/// let router: Router<()> = git_router!(
///     "/git-upload-pack" => get::<_, ((),), ()>(|| async {})
///     "/git-receive-pack" => get::<_, ((),), ()>(|| async {})
/// );
/// # }
/// ```
#[macro_export]
macro_rules! git_router {
    ($($path:tt => $endpoint:expr)+) => {
            axum::Router::new()
        $(
            .route(const { const_format::concatcp!("/{public_key}/{repo_name}", $path) }, $endpoint)
        )+
    };
}

/// Creates a router for git-related endpoints
pub fn router(config: &RelayConfig) -> Router {
    let mut router = git_router!(
        "/git-upload-pack" => post(upload_pack)
        "/git-receive-pack" => post(receive_pack)
        "/info/refs" => get(info_refs)
        "/HEAD" => get(get_file_content::<false>)
        "/objects/info/packs" => get(get_file_content::<true>)
        "/objects/info/{*rest}" => get(get_file_content::<false>)
        "/objects/pack/{pack}" => get(get_file_content::<true>)
    );

    if let Some(max_reqs) = config.grasp.max_reqs {
        router = router.layer(ConcurrencyLimitLayer::new(max_reqs.into()));
    }

    if let Some(timeout) = config.grasp.req_timeout {
        router = router.layer(TimeoutLayer::new(Duration::from_secs(timeout.into())));
    }

    router
}
