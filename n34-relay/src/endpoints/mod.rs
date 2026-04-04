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

use std::{
    net::SocketAddr,
    sync::{Arc, OnceLock},
};

use axum::{
    Extension,
    extract::{ConnectInfo, FromRequest, Request},
    response::{Html, IntoResponse, Response},
};

use crate::{
    ext_traits::HeaderMapExt,
    raw_websocket::RawSocketUpgrade,
    router_state::RouterState,
    utils,
};

/// Relay Information Document (NIP-11)
mod nip11;
/// Relay Management API (NIP-86)
mod nip86;

static HOME_PAGE_CONTENT: OnceLock<String> = OnceLock::new();

/// Handles incoming requests.
///
/// routing them to the landing page, management API, or upgrading to a
/// WebSocket connection for the relay.
pub async fn main_handler(
    Extension(state): Extension<Arc<RouterState>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    req: Request,
) -> Response {
    if req.headers().is_ws_upgrade() {
        match RawSocketUpgrade::from_request(req, &()).await {
            Ok(ws) => handle_ws(state, addr, ws),
            Err(err) => {
                (
                    axum::http::StatusCode::BAD_REQUEST,
                    format!("Failed to upgrade the connection: {err}"),
                )
                    .into_response()
            }
        }
    } else if req.headers().is_nip86_req() {
        nip86::main_nip86_handler(state, req).await.into_response()
    } else if req.headers().is_nip11_req() {
        self::nip11::handler(state).await
    } else {
        Html(
            HOME_PAGE_CONTENT
                .get_or_init(utils::homepage_content)
                .as_str(),
        )
        .into_response()
    }
}


/// Pass the websocket connection to the relay
pub fn handle_ws(state: Arc<RouterState>, addr: SocketAddr, ws: RawSocketUpgrade) -> Response {
    ws.on_upgrade(async move |socket| {
        if let Err(err) = state.relay.take_connection(socket, addr).await {
            tracing::error!(addr = %addr, error = %err, "Failed to handle WebSocket connection");
        }
    })
}
