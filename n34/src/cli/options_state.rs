// n34 - A CLI to interact with NIP-34 and other stuff related to codes in nostr
// Copyright (C) 2025 Awiteb <a@4rs.nl>
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://gnu.org/licenses/gpl-3.0.html>.

use std::{
    net::{Ipv4Addr, SocketAddr, SocketAddrV4},
    time::Duration,
};

use nostr_browser_signer_proxy::{BrowserSignerProxy, BrowserSignerProxyOptions};

/// The default socket address used for the NIP-07 signer proxy, set to
/// localhost on port 51034.
pub const DEFAULT_NIP07_PROXY_ADDR: SocketAddr =
    SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 51034));

/// How long to wait for the proxy response (3 minutes).
pub const BROWSER_SIGNER_PROXY_TIMEOUT: Duration = Duration::from_secs(60 * 3);


/// Represents the state used for CLI options.
pub struct OptionsState {
    /// The browser signer proxy, will be used if `--nip07` is enabled
    pub browser_signer_proxy: BrowserSignerProxy,
}

impl Default for OptionsState {
    fn default() -> Self {
        Self {
            browser_signer_proxy: default_browser_signer_proxy(),
        }
    }
}

/// Build the default browser signer proxy
#[inline]
fn default_browser_signer_proxy() -> BrowserSignerProxy {
    BrowserSignerProxy::new(
        BrowserSignerProxyOptions::default()
            .timeout(BROWSER_SIGNER_PROXY_TIMEOUT)
            .ip_addr(DEFAULT_NIP07_PROXY_ADDR.ip())
            .port(DEFAULT_NIP07_PROXY_ADDR.port()),
    )
}
