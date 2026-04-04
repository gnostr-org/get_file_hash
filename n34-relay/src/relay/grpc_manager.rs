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

use std::{collections::BTreeSet, sync::Arc};

use either::Either;
use nostr::{event::Event, filter::Filter, message::MachineReadablePrefix, util::BoxedFuture};
use nostr_relay_builder::builder::{
    QueryPolicy,
    QueryPolicyResult,
    WritePolicy,
    WritePolicyResult,
};
use tonic::transport::Channel;

use self::plugins_api::{
    Empty,
    PluginInfo,
    PluginPriority,
    PluginRequest,
    PluginType,
    plugin_request::PluginRequestBody,
    plugin_response::PluginResponseBody,
    plugins_service_client::PluginsServiceClient,
};
use crate::{relay::plugins_manager::PluginsManagerTrait, relay_config::RelayConfig};

mod plugins_api {
    tonic::include_proto!("plugins");
}

type GrpcResult<T> = Result<T, GrpcError>;
type PluginClient = PluginsServiceClient<Channel>;

/// gRPC errors
#[derive(Debug, thiserror::Error)]
pub enum GrpcError {
    #[error("Transport: {0:?}")]
    Transport(#[from] tonic::transport::Error),
    #[error("Call error: {0}")]
    CallErr(String),
    #[error("A plugin service without any plugin {0}")]
    NoPlugins(hyper::Uri),
    #[error("Unknown plugin type: {0}")]
    UnknownPluginType(i32),
    #[error("Unknown plugin priority: {0}")]
    UnknownPluginPriority(i32),
}

/// Represents a gRPC plugin.
#[derive(Debug)]
pub struct GrpcPlugin {
    /// The name of the plugin.
    name:     String,
    /// Indicates whether the plugin is for writing or querying.
    is_write: bool,
    /// Plugin priority, `all` or `any`
    is_all:   bool,
}

/// Handles gRPC plugin service.
#[derive(Debug)]
pub struct GrpcPluginService {
    /// Service uri
    pub uri:               hyper::Uri,
    /// The gRPC client used to call the `RunPlugin` RPC.
    pub grpc_client:       PluginClient,
    /// Write plugins of type `all`
    pub all_write_plugins: Vec<GrpcPlugin>,
    /// Write plugins of type `any`
    pub any_write_plugins: Vec<GrpcPlugin>,
    /// Query plugins of type `all`
    pub all_query_plugins: Vec<GrpcPlugin>,
    /// Query plugins of type `any`
    pub any_query_plugins: Vec<GrpcPlugin>,
}

/// Manages multiple gRPC plugin services.
///
/// This is safe to clone as it uses an internal reference counter.
#[derive(Debug, Clone)]
pub struct GrpcPluginsManager {
    /// A collection of gRPC services, each of which contains multiple plugins.
    pub services: Arc<Vec<GrpcPluginService>>,
}

impl GrpcPluginService {
    /// Create a new service and connect to it
    pub async fn new(service_uri: hyper::Uri) -> GrpcResult<Self> {
        tracing::debug!(service_uri = %service_uri, "connecting to a plugins service");
        let mut client = PluginClient::connect(service_uri.clone()).await?;

        tracing::debug!(service_uri = %service_uri, "calling `GetPlugins` RPC");
        let mut plugins = client
            .get_plugins(Empty {})
            .await
            .map_err(|err| GrpcError::CallErr(err.to_string()))?
            .into_inner()
            .plugins;

        // Sort and remove all duplicates
        plugins.sort_unstable_by_key(|plugin| plugin.name.clone());
        plugins.dedup_by_key(|plugin: &mut PluginInfo| plugin.name.clone());
        tracing::debug!(service_uri = %service_uri, "received {} plugins: {plugins:?}", plugins.len());

        if plugins.is_empty() {
            return Err(GrpcError::NoPlugins(service_uri));
        }

        let mut all_write_plugins = Vec::new();
        let mut any_write_plugins = Vec::new();
        let mut all_query_plugins = Vec::new();
        let mut any_query_plugins = Vec::new();

        for plugin in plugins.into_iter().map(GrpcPlugin::try_from) {
            let plugin = plugin?;
            match (plugin.is_write, plugin.is_all) {
                (true, true) => all_write_plugins.push(plugin),
                (true, false) => any_write_plugins.push(plugin),
                (false, true) => all_query_plugins.push(plugin),
                (false, false) => any_query_plugins.push(plugin),
            }
        }

        Ok(Self {
            uri: service_uri,
            grpc_client: client,
            all_write_plugins,
            any_write_plugins,
            all_query_plugins,
            any_query_plugins,
        })
    }
}

impl GrpcPluginsManager {
    /// Initializes a gRPC plugins manager using the provided services.
    ///
    /// Connects to each service to retrieve its available plugins.
    pub async fn with_services(services: &[hyper::Uri]) -> GrpcResult<Self> {
        let mut manager_services = Vec::new();

        for service in services {
            manager_services.push(GrpcPluginService::new(service.clone()).await?);
        }

        Ok(Self {
            services: Arc::new(manager_services),
        })
    }

    /// Gets the manager if gRPC plugins are configured, otherwise returns None.
    /// Logs an error if initialization fails.
    pub async fn maybe_manager(config: &RelayConfig) -> Option<Self> {
        if config.plugins.grpc.is_empty() {
            tracing::info!("gRPC plugins list is empty, skipping connection");
            return None;
        }

        match GrpcPluginsManager::with_services(&config.plugins.grpc).await {
            Ok(manager) => {
                tracing::info!("Successfully initialized gRPC plugins manager");
                Some(manager)
            }
            Err(err) => {
                tracing::error!("Failed to initialize gRPC plugins. Error: {}", err);
                None
            }
        }
    }
}

impl GrpcPlugin {
    /// Run the plugin
    pub async fn run(
        &self,
        uri: &hyper::Uri,
        client: &mut PluginClient,
        body: Either<&Event, &Filter>,
    ) -> Result<(), String> {
        let request = if self.is_write {
            assert!(body.is_left());
            write_plugin_request(&self.name, body.unwrap_left())
        } else {
            assert!(body.is_right());
            query_plugin_request(&self.name, body.unwrap_right().clone())
        };

        match client
            .run_plugin(request)
            .await
            .map(|r| r.into_inner().plugin_response_body)
        {
            Ok(Some(PluginResponseBody::Accept(..))) => Ok(()),
            Ok(Some(PluginResponseBody::RejectMsg(msg))) => Err(msg),
            Ok(None) => {
                tracing::warn!(service_uri = %uri, plugin_name = %self.name, "plugin returns none body");
                Ok(())
            }
            Err(err) => {
                tracing::error!(service_uri = %uri, plugin_name = %self.name, "plugin returns error: {err:?}");
                Ok(())
            }
        }
    }
}

impl GrpcPluginService {
    /// Executes the provided `all_plugins` and `any_plugins` with the given
    /// `body`.
    ///
    /// The `all_plugins` must all accept the `body` (either an `Event` or
    /// `Filter`). If they do, the `any_plugins` are then checked. If at
    /// least one `any_plugin` accepts the `body`, the function returns
    /// `Ok(())`. Otherwise, it returns an `Err` with the rejection message
    /// from the first `any_plugin` that rejected the `body`.
    async fn run_plugins(
        &self,
        body: Either<&Event, &Filter>,
        all_plugins: &[GrpcPlugin],
        any_plugins: &[GrpcPlugin],
    ) -> Result<(), String> {
        if all_plugins.is_empty() && any_plugins.is_empty() {
            return Ok(());
        }

        let mut client = self.grpc_client.clone();

        // All `all_plugins` must accept the `body` to proceed.
        for plugin in all_plugins {
            plugin.run(&self.uri, &mut client, body).await?
        }

        // If there are no `any_plugins`, the `body` is accepted.
        if any_plugins.is_empty() {
            return Ok(());
        }

        let mut reject_msg = None;
        for plugin in any_plugins {
            match plugin.run(&self.uri, &mut client, body).await {
                // If any `any_plugin` accepts the `body`, return `Ok(())`.
                Ok(()) => return Ok(()),
                // Store the rejection message from the first `any_plugin` that rejects the `body`.
                Err(msg) if reject_msg.is_none() => reject_msg = Some(msg),
                _ => (),
            }
        }

        // If no `any_plugin` accepts the `body`, return the first rejection message.
        Err(reject_msg.unwrap())
    }

    /// Executes the write plugins for the given `event`.
    ///
    /// Returns `Ok(())` if all `all_plugins` accept the `event` and at least
    /// one `any_plugin` accepts it. Otherwise, returns `Err(reject_msg)`
    /// with the rejection message.
    pub async fn run_write_plugins(&self, event: &Event) -> Result<(), String> {
        self.run_plugins(
            Either::Left(event),
            &self.all_write_plugins,
            &self.any_write_plugins,
        )
        .await
    }

    /// Executes the query plugins for the given `event`.
    ///
    /// Returns `Ok(())` if all `all_plugins` accept the `event` and at least
    /// one `any_plugin` accepts it. Otherwise, returns `Err(reject_msg)`
    /// with the rejection message.
    pub async fn run_query_plugins(&self, query: &Filter) -> Result<(), String> {
        self.run_plugins(
            Either::Right(query),
            &self.all_query_plugins,
            &self.any_query_plugins,
        )
        .await
    }
}

impl TryFrom<PluginInfo> for GrpcPlugin {
    type Error = GrpcError;

    fn try_from(plugin: PluginInfo) -> Result<Self, Self::Error> {
        tracing::debug!(
            plugin_name = %plugin.name,
            plugin_type = %plugin.plugin_type,
            plugin_priority = %plugin.priority,
            "processing plugin info"
        );

        let is_write = PluginType::try_from(plugin.plugin_type)
            .map_err(|_| GrpcError::UnknownPluginType(plugin.plugin_type))?
            == PluginType::Write;

        let is_all = PluginPriority::try_from(plugin.priority)
            .map_err(|_| GrpcError::UnknownPluginPriority(plugin.priority))?
            == PluginPriority::All;

        Ok(Self {
            name: plugin.name,
            is_write,
            is_all,
        })
    }
}

impl WritePolicy for GrpcPluginsManager {
    fn admit_event<'a>(
        &'a self,
        event: &'a Event,
        _: &'a std::net::SocketAddr,
    ) -> BoxedFuture<'a, WritePolicyResult> {
        Box::pin(async {
            for service in self.services.iter() {
                if let Err(reject_msg) = service.run_write_plugins(event).await {
                    tracing::debug!(service_url = %service.uri, "event rejected: {}", event.id);
                    return WritePolicyResult::reject(MachineReadablePrefix::Blocked, reject_msg);
                }
            }

            WritePolicyResult::Accept
        })
    }
}

impl QueryPolicy for GrpcPluginsManager {
    fn admit_query<'a>(
        &'a self,
        query: &'a Filter,
        _: &'a std::net::SocketAddr,
    ) -> BoxedFuture<'a, QueryPolicyResult> {
        Box::pin(async move {
            for service in self.services.iter() {
                if let Err(reject_msg) = service.run_query_plugins(query).await {
                    tracing::debug!(service_url = %service.uri, "query rejected: {query:?}");
                    return QueryPolicyResult::reject(MachineReadablePrefix::Blocked, reject_msg);
                }
            }

            QueryPolicyResult::Accept
        })
    }
}

impl PluginsManagerTrait for GrpcPluginsManager {}

/// Make a new write plugin request
fn write_plugin_request(plugin_name: &str, event: &Event) -> PluginRequest {
    let tags = event
        .tags
        .iter()
        .map(|tag| {
            plugins_api::Tag {
                tag_kind: tag.kind().as_str().to_owned(),
                values:   tag.as_slice()[1..].to_vec(),
            }
        })
        .collect();

    PluginRequest {
        plugin_name:         plugin_name.to_owned(),
        plugin_request_body: Some(PluginRequestBody::Event(plugins_api::Event {
            tags,
            id: event.id.to_hex(),
            public_key: event.pubkey.to_hex(),
            created_at: event.created_at.as_secs(),
            kind: event.kind.as_u16() as u32,
            content: event.content.clone(),
            signature: event.sig.to_string(),
        })),
    }
}

/// Make a new query plugin request
fn query_plugin_request(plugin_name: &str, filter: Filter) -> PluginRequest {
    let tags = filter
        .generic_tags
        .iter()
        .map(|(tag, values)| {
            plugins_api::Tag {
                tag_kind: tag.as_str().to_owned(),
                values:   values.clone().into_iter().collect(),
            }
        })
        .collect();

    PluginRequest {
        plugin_name:         plugin_name.to_owned(),
        plugin_request_body: Some(PluginRequestBody::Filter(plugins_api::Filter {
            tags,
            ids: map_set(filter.ids, |id| id.to_hex()),
            authors: map_set(filter.authors, |pkey| pkey.to_hex()),
            kinds: map_set(filter.kinds, |kind| kind.as_u16() as u32),
            since: filter.since.map(|t| t.as_secs()),
            until: filter.until.map(|t| t.as_secs()),
            limit: filter
                .limit
                .map(|limit| limit.try_into().unwrap_or(u64::MAX)),
        })),
    }
}

fn map_set<T, R>(option_set: Option<BTreeSet<T>>, map_fn: impl FnMut(T) -> R) -> Vec<R> {
    option_set
        .map(|set| set.into_iter().map(map_fn).collect())
        .unwrap_or_default()
}
