// Copyright 2025 Thomas Profelt
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the “Software”), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in
// all copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED “AS IS”, WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

// This implementation is derived from the original work authored by Thomas
// Profelt in the `axum-raw-websocket` library.
//
// Source: https://github.com/tompro/axum-raw-websocket/blob/master/src/lib.rs
// Licensed under the MIT License.
//
// Note: Only this specific file is licensed under MIT. The remainder of the
// project is licensed under AGPLv3.

use std::future::Future;

use axum::body::Body;
use axum::extract::FromRequestParts;
use axum::extract::ws::rejection::InvalidProtocolPseudoheader;
use axum::extract::ws::rejection::{
    ConnectionNotUpgradable,
    InvalidConnectionHeader,
    InvalidUpgradeHeader,
    InvalidWebSocketVersionHeader,
    MethodNotConnect,
    MethodNotGet,
    WebSocketKeyHeaderMissing,
    WebSocketUpgradeRejection,
};
use axum::http::{
    Method,
    StatusCode,
    Version,
    header::{self, HeaderMap, HeaderName, HeaderValue},
    request::Parts,
};
use axum::{Error, body::Bytes, response::Response};
use hyper::upgrade::Upgraded;
use hyper_util::rt::TokioIo;
use sha1::{Digest, Sha1};

/// This websocket upgrade is based on the axum integrated one
/// [WebSocketUpgrade](axum::extract::ws::WebSocketUpgrade).
///
/// The main difference is that it will onvoke the on_upgrade callback with the
/// raw socket which allow the socket to be used by other libraries than the
/// default tokio-tungstenite.
///
/// For HTTP/1.1 requests, this extractor requires the request method to be
/// `GET`; in later versions, `CONNECT` is used instead.
/// To support both, it should be used with [`any`](axum::routing::any).
///
/// See the [module docs](axum::extract::ws) for an example.
///
/// [`MethodFilter`]: axum::routing::MethodFilter
#[cfg_attr(docsrs, doc(cfg(feature = "ws")))]
pub struct RawSocketUpgrade<F = DefaultOnFailedUpgrade> {
    /// `None` if HTTP/2+ WebSockets are used.
    sec_websocket_key:      Option<HeaderValue>,
    on_upgrade:             hyper::upgrade::OnUpgrade,
    on_failed_upgrade:      F,
    sec_websocket_protocol: Option<HeaderValue>,
}

impl<F> std::fmt::Debug for RawSocketUpgrade<F> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RelayUpgrade")
            .field("sec_websocket_key", &self.sec_websocket_key)
            .field("sec_websocket_protocol", &self.sec_websocket_protocol)
            .finish_non_exhaustive()
    }
}

impl<F> RawSocketUpgrade<F> {
    #[allow(dead_code)]
    pub fn on_failed_upgrade<C>(self, callback: C) -> RawSocketUpgrade<C>
    where
        C: OnFailedUpgrade,
    {
        RawSocketUpgrade {
            sec_websocket_key:      self.sec_websocket_key,
            on_upgrade:             self.on_upgrade,
            on_failed_upgrade:      callback,
            sec_websocket_protocol: self.sec_websocket_protocol,
        }
    }

    /// Finalize upgrading the connection and call the provided callback with
    /// the stream.
    #[must_use = "to set up the WebSocket connection, this response must be returned"]
    pub fn on_upgrade<C, Fut>(self, callback: C) -> Response
    where
        C: FnOnce(TokioIo<Upgraded>) -> Fut + Send + 'static,
        Fut: Future<Output = ()> + Send + 'static,
        F: OnFailedUpgrade,
    {
        let on_upgrade = self.on_upgrade;
        let on_failed_upgrade = self.on_failed_upgrade;

        tokio::spawn(async move {
            let upgraded = match on_upgrade.await {
                Ok(upgraded) => upgraded,
                Err(err) => {
                    on_failed_upgrade.call(Error::new(err));
                    return;
                }
            };
            let upgraded: TokioIo<Upgraded> = TokioIo::new(upgraded);
            callback(upgraded).await;
        });


        if let Some(sec_websocket_key) = &self.sec_websocket_key {
            // If `sec_websocket_key` was `Some`, we are using HTTP/1.1.

            #[allow(clippy::declare_interior_mutable_const)]
            const UPGRADE: HeaderValue = HeaderValue::from_static("upgrade");
            #[allow(clippy::declare_interior_mutable_const)]
            const WEBSOCKET: HeaderValue = HeaderValue::from_static("websocket");

            Response::builder()
                .status(StatusCode::SWITCHING_PROTOCOLS)
                .header(header::CONNECTION, UPGRADE)
                .header(header::UPGRADE, WEBSOCKET)
                .header(
                    header::SEC_WEBSOCKET_ACCEPT,
                    sign(sec_websocket_key.as_bytes()),
                )
                .body(Body::empty())
                .unwrap()
        } else {
            Response::new(Body::empty())
        }
    }
}

/// What to do when a connection upgrade fails.
///
/// See [`RawSocketUpgrade::on_failed_upgrade`] for more details.
pub trait OnFailedUpgrade: Send + 'static {
    /// Call the callback.
    fn call(self, error: Error);
}

impl<F> OnFailedUpgrade for F
where
    F: FnOnce(Error) + Send + 'static,
{
    fn call(self, error: Error) {
        self(error)
    }
}

/// The default `OnFailedUpgrade` used by `RawSocketUpgrade`.
///
/// It simply ignores the error.
#[non_exhaustive]
#[derive(Debug)]
pub struct DefaultOnFailedUpgrade;

impl OnFailedUpgrade for DefaultOnFailedUpgrade {
    #[inline]
    fn call(self, _error: Error) {}
}

impl<S> FromRequestParts<S> for RawSocketUpgrade<DefaultOnFailedUpgrade>
where
    S: Send + Sync,
{
    type Rejection = WebSocketUpgradeRejection;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let sec_websocket_key = if parts.version <= Version::HTTP_11 {
            if parts.method != Method::GET {
                return Err(WebSocketUpgradeRejection::MethodNotGet(
                    MethodNotGet::default(),
                ));
            }

            if !header_contains(&parts.headers, header::CONNECTION, "upgrade") {
                return Err(WebSocketUpgradeRejection::InvalidConnectionHeader(
                    InvalidConnectionHeader::default(),
                ));
            }

            if !header_eq(&parts.headers, header::UPGRADE, "websocket") {
                return Err(WebSocketUpgradeRejection::InvalidUpgradeHeader(
                    InvalidUpgradeHeader::default(),
                ));
            }

            Some(
                parts
                    .headers
                    .get(header::SEC_WEBSOCKET_KEY)
                    .ok_or(WebSocketUpgradeRejection::WebSocketKeyHeaderMissing(
                        WebSocketKeyHeaderMissing::default(),
                    ))?
                    .clone(),
            )
        } else {
            if parts.method != Method::CONNECT {
                return Err(WebSocketUpgradeRejection::MethodNotConnect(
                    MethodNotConnect::default(),
                ));
            }

            if parts
                .extensions
                .get::<hyper::ext::Protocol>()
                .is_none_or(|p| p.as_str() != "websocket")
            {
                return Err(WebSocketUpgradeRejection::InvalidProtocolPseudoheader(
                    InvalidProtocolPseudoheader::default(),
                ));
            }

            None
        };

        if !header_eq(&parts.headers, header::SEC_WEBSOCKET_VERSION, "13") {
            return Err(WebSocketUpgradeRejection::InvalidWebSocketVersionHeader(
                InvalidWebSocketVersionHeader::default(),
            ));
        }

        let on_upgrade = parts
            .extensions
            .remove::<hyper::upgrade::OnUpgrade>()
            .ok_or(WebSocketUpgradeRejection::ConnectionNotUpgradable(
                ConnectionNotUpgradable::default(),
            ))?;

        let sec_websocket_protocol = parts.headers.get(header::SEC_WEBSOCKET_PROTOCOL).cloned();

        Ok(Self {
            sec_websocket_key,
            on_upgrade,
            sec_websocket_protocol,
            on_failed_upgrade: DefaultOnFailedUpgrade,
        })
    }
}

fn header_eq(headers: &HeaderMap, key: HeaderName, value: &'static str) -> bool {
    if let Some(header) = headers.get(&key) {
        header.as_bytes().eq_ignore_ascii_case(value.as_bytes())
    } else {
        false
    }
}

fn header_contains(headers: &HeaderMap, key: HeaderName, value: &'static str) -> bool {
    let header = if let Some(header) = headers.get(&key) {
        header
    } else {
        return false;
    };

    if let Ok(header) = std::str::from_utf8(header.as_bytes()) {
        header.to_ascii_lowercase().contains(value)
    } else {
        false
    }
}

fn sign(key: &[u8]) -> HeaderValue {
    use base64::engine::Engine as _;

    let mut sha1 = Sha1::default();
    sha1.update(key);
    sha1.update(b"258EAFA5-E914-47DA-95CA-C5AB0DC85B11");
    let b64 = Bytes::from(base64::engine::general_purpose::STANDARD.encode(sha1.finalize()));
    HeaderValue::from_maybe_shared(b64).expect("base64 is a valid value")
}
