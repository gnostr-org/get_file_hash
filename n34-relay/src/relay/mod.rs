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

use nostr_database::NostrDatabase;
use nostr_relay_builder::{
    LocalRelay,

    builder::{LocalRelayBuilderNip42, RateLimit},
};

pub use self::{grpc_manager::GrpcError, rhai_manager::RhaiPluginsError};
use crate::relay_config::RelayConfig;

/// A plugin to check the event size.
mod event_size_plugin;
/// Core GRASP (Git Relays Authorized via Signed-Nostr Proofs) implementation
mod grasp;
/// gRPC plugins manager
mod grpc_manager;
/// Plugins manager
mod plugins_manager;
/// Rhai plugins manager
mod rhai_manager;
/// Plugins that manages a whitelist and blacklist
mod whitelist_plugins;

/// Middlewares state
pub struct MiddlewareState {
    /// The relay config
    pub config: Arc<RelayConfig>,
}

impl MiddlewareState {
    /// Construct a new middleware state
    pub fn new(config: Arc<RelayConfig>) -> Self {
        Self { config }
    }
}

/// Build a relay from the config
pub async fn build_relay(config: Arc<RelayConfig>, relay_db: Arc<dyn NostrDatabase>) -> LocalRelay {
    let mut plugins_builder = plugins_manager::PluginsManagerBuilder::with_middlewares_state(
        Arc::new(MiddlewareState::new(Arc::clone(&config))),
    )
    .add_all(event_size_plugin::EventSizePlugin(
        config.relay.max_event_size.into(),
    ))
    .add_all(whitelist_plugins::PubKeyBlacklist(Arc::clone(
        &config.relay.blacklist,
    )))
    .add_all(whitelist_plugins::KindBlacklist(Arc::clone(
        &config.relay.disallowed_kinds,
    )))
    .add_all(whitelist_plugins::KindWhitelist(Arc::clone(
        &config.relay.allowed_kinds,
    )))
    .add_any(whitelist_plugins::PubKeyWhiteList(Arc::clone(
        &config.relay.whitelist,
    )))
    .add_any(whitelist_plugins::MentionedPubKey(Arc::clone(
        &config.relay.whitelist,
    )))
    .add_plugins_manager(grpc_manager::GrpcPluginsManager::maybe_manager(&config).await)
    .add_plugins_manager(rhai_manager::maybe_manager(&config).await);

    if config.grasp.enable {
        plugins_builder = plugins_builder
            .add_all(grasp::plugins::ValidateRepoState)
            .add_all(grasp::plugins::ValidateRepoEvent)
            .add_all(grasp::plugins::RejectRepoState::new(Arc::clone(&relay_db)))
            .add_all(grasp::plugins::GraspRepo::new(&config.relay.domain))
            .add_any(grasp::plugins::AcceptMention::new(Arc::clone(&relay_db)))
            .add_middleware(grasp::middlewares::repo_creator);
    }

    let plugins = plugins_builder.build();
    let mut relay_builder = LocalRelay::builder()
        .addr(config.net.ip)
        .port(config.net.port)
        .auth_dm(true)
        .min_pow(config.relay.min_pow)
        .rate_limit(RateLimit::from(config.ratelimit.clone()))
        .max_filter_limit(config.relay.max_limit.into())
        .default_filter_limit(config.relay.default_limit.into())
        .max_subid_length(config.relay.max_subid_length.into())
        .database(relay_db)
        .write_policy(plugins.clone())
        .query_policy(plugins);

    if config.relay.nip42 {
        relay_builder = relay_builder.nip42(LocalRelayBuilderNip42::read_and_write());
    }

    if let Some(max_connections) = config.relay.max_connections {
        relay_builder = relay_builder.max_connections(max_connections)
    }

    relay_builder.build()
}
