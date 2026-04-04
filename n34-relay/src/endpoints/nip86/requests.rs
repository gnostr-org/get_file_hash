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

use parking_lot::RwLock;
use serde::Deserialize;
use strum::VariantNames;

use crate::{
    endpoints::nip86::{
        errors::ApiResult,
        params::{KindParam, PubKeyWithReason, PublicKeyParam, StringParam, UrlParam},
        responses::Nip86Result,
    },
    router_state::RouterState,
};

type EmptyParams = [u8; 0];

/// The request body of the API
#[derive(Deserialize, VariantNames)]
#[serde(rename_all = "lowercase", tag = "method")]
#[strum(serialize_all = "lowercase")]
#[allow(dead_code)] // `EmptyParams` will never read
pub enum Nip86Request {
    SupportedMethods { params: EmptyParams },
    ChangeRelayName { params: StringParam },
    ChangeRelayDescription { params: StringParam },
    ChangeRelayIcon { params: UrlParam },
    ChangeRelayBanner { params: UrlParam },
    AllowPubkey { params: PubKeyWithReason },
    AllowKind { params: KindParam },
    AddAdmin { params: PublicKeyParam },
    BanPubkey { params: PubKeyWithReason },
    DisallowKind { params: KindParam },
    RemoveAdmin { params: PublicKeyParam },
    ListAllowedPubkeys { params: EmptyParams },
    ListBannedPubkeys { params: EmptyParams },
    ListAllowedKinds { params: EmptyParams },
    ListDisallowedKinds { params: EmptyParams },
    ListAdmins { params: EmptyParams },
}

impl Nip86Request {
    /// Processes the request and returns the response.
    pub async fn run(self, state: Arc<RouterState>) -> ApiResult<Nip86Result> {
        match self {
            Nip86Request::SupportedMethods { .. } => {
                Ok(Nip86Result::SupportedMethods(Self::VARIANTS))
            }
            Nip86Request::ChangeRelayName {
                params: StringParam(new_name),
            } => {
                *state.config.nip11.name.write() = Some(new_name);
                Ok(Nip86Result::True)
            }
            Nip86Request::ChangeRelayDescription {
                params: StringParam(new_description),
            } => {
                *state.config.nip11.description.write() = Some(new_description);
                Ok(Nip86Result::True)
            }
            Nip86Request::ChangeRelayIcon {
                params: UrlParam(new_icon),
            } => {
                *state.config.nip11.icon.write() = Some(new_icon);
                Ok(Nip86Result::True)
            }
            Nip86Request::ChangeRelayBanner {
                params: UrlParam(new_banner),
            } => {
                *state.config.nip11.banner.write() = Some(new_banner);
                Ok(Nip86Result::True)
            }
            Nip86Request::AllowPubkey { params } => {
                list_value(
                    Some(Arc::clone(&state.config.relay.whitelist)),
                    Some(Arc::clone(&state.config.relay.blacklist)),
                    params.pubkey,
                )
            }
            Nip86Request::BanPubkey { params } => {
                list_value(
                    Some(Arc::clone(&state.config.relay.blacklist)),
                    Some(Arc::clone(&state.config.relay.whitelist)),
                    params.pubkey,
                )
            }
            Nip86Request::AllowKind {
                params: KindParam(kind),
            } => {
                list_value(
                    Some(Arc::clone(&state.config.relay.allowed_kinds)),
                    Some(Arc::clone(&state.config.relay.disallowed_kinds)),
                    kind,
                )
            }
            Nip86Request::DisallowKind {
                params: KindParam(kind),
            } => {
                list_value(
                    Some(Arc::clone(&state.config.relay.disallowed_kinds)),
                    Some(Arc::clone(&state.config.relay.allowed_kinds)),
                    kind,
                )
            }
            Nip86Request::AddAdmin {
                params: PublicKeyParam(pkey),
            } => list_value(Some(Arc::clone(&state.config.relay.admins)), None, pkey),
            Nip86Request::RemoveAdmin {
                params: PublicKeyParam(pkey),
            } => list_value(None, Some(Arc::clone(&state.config.relay.admins)), pkey),
            Nip86Request::ListAllowedPubkeys { .. } => {
                list_values(Arc::clone(&state.config.relay.whitelist), |pkeys| {
                    Nip86Result::PublicKeysAndReason(
                        pkeys.into_iter().map(PubKeyWithReason::from).collect(),
                    )
                })
            }
            Nip86Request::ListBannedPubkeys { .. } => {
                list_values(Arc::clone(&state.config.relay.blacklist), |pkeys| {
                    Nip86Result::PublicKeysAndReason(
                        pkeys.into_iter().map(PubKeyWithReason::from).collect(),
                    )
                })
            }
            Nip86Request::ListAllowedKinds { .. } => {
                list_values(
                    Arc::clone(&state.config.relay.allowed_kinds),
                    Nip86Result::Kinds,
                )
            }
            Nip86Request::ListDisallowedKinds { .. } => {
                list_values(
                    Arc::clone(&state.config.relay.disallowed_kinds),
                    Nip86Result::Kinds,
                )
            }
            Nip86Request::ListAdmins { .. } => {
                list_values(
                    Arc::clone(&state.config.relay.admins),
                    Nip86Result::PublicKeys,
                )
            }
        }
    }

    /// Checks if the request can only be performed by the super admin (the
    /// first admin in the list).
    #[inline]
    pub fn only_superadmin(&self) -> bool {
        matches!(
            self,
            Self::AddAdmin { .. } | Self::RemoveAdmin { .. } | Self::ListAdmins { .. }
        )
    }
}

/// Adds a value to either the whitelist or blacklist, perhaps :).
fn list_value<T: PartialEq>(
    add_to: Option<Arc<RwLock<Vec<T>>>>,
    remove_from: Option<Arc<RwLock<Vec<T>>>>,
    value: T,
) -> ApiResult<Nip86Result> {
    if let Some(remove_from) = remove_from {
        let mut remove_from_lock = remove_from.write();
        if remove_from_lock.contains(&value) {
            remove_from_lock.retain(|p| p != &value)
        }
    }

    if let Some(add_to) = add_to {
        let mut add_to_lock = add_to.write();
        if !add_to_lock.contains(&value) {
            add_to_lock.push(value);
        }
    }

    Ok(Nip86Result::True)
}

/// List values. Whitelist or blacklist, perhaps :)
fn list_values<T: Copy>(
    list_from: Arc<RwLock<Vec<T>>>,
    result_fn: impl FnOnce(Vec<T>) -> Nip86Result,
) -> ApiResult<Nip86Result> {
    Ok(result_fn(list_from.read().iter().copied().collect()))
}
