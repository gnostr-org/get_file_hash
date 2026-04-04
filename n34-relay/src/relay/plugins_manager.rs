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

use std::{fmt, net::SocketAddr, sync::Arc};

use either::Either;
use nostr::{event::Event, filter::Filter, util::BoxedFuture};
use nostr_relay_builder::builder::{
    QueryPolicy,
    QueryPolicyResult,
    WritePolicy,
    WritePolicyResult,
};

/// Middleware function type
type MiddlewareFn<S> = for<'a> fn(Either<&'a Filter, &'a Event>, S) -> BoxedFuture<'a, ()>;

/// Trait that plugins managers must implement, combining WritePolicy and
/// QueryPolicy capabilities.
pub trait PluginsManagerTrait: WritePolicy + QueryPolicy {}

/// Manages plugins that validate events and queries.
///
/// Plugins are divided into two groups:
/// - `all`: All plugins must accept the event/query
/// - `any`: At least one plugin must accept the event/query
///
/// First checks all `all` plugins, then checks `any` plugins. For example:
/// - Blacklists (keys, kinds, sizes) would be `all` plugins
/// - Whitelists could be either, with `any` allowing alternatives (like
///   whitelisted mentions)
#[derive(Clone)]
pub struct PluginsManager<S>
where
    S: Clone,
{
    /// Plugins where any one can accept the event/query
    any_plugins:      Arc<Vec<Box<dyn RelayPlugin>>>,
    /// Plugins where all must accept the event/query
    all_plugins:      Arc<Vec<Box<dyn RelayPlugin>>>,
    /// Plugins managers, each one have their own plugins
    plugins_managers: Arc<Vec<Box<dyn PluginsManagerTrait>>>,
    /// Relay middlewares
    middlewares:      Arc<Vec<MiddlewareFn<S>>>,
    /// Middleware state
    middleware_state: S,
}

/// Builder of [PluginsManager]
pub struct PluginsManagerBuilder<S>
where
    S: Clone,
{
    /// Plugins where any one can accept the event/query
    any_plugins:      Vec<Box<dyn RelayPlugin>>,
    /// Plugins where all must accept the event/query
    all_plugins:      Vec<Box<dyn RelayPlugin>>,
    /// Other plugins managers
    plugins_managers: Vec<Box<dyn PluginsManagerTrait>>,
    /// Relay middlewares
    middlewares:      Vec<MiddlewareFn<S>>,
    /// Middleware state
    middleware_state: S,
}

/// Plugin interface for validating relay events and queries
pub trait RelayPlugin: Send + Sync {
    /// Determines whether the event should be accepted or rejected.
    ///
    /// Returns `Some(WritePolicyResult)` to accept or reject, or `None` to take
    /// no action.
    #[allow(unused_variables)]
    fn check_event<'a>(&'a self, event: &'a Event) -> BoxedFuture<'a, Option<WritePolicyResult>> {
        Box::pin(async { None })
    }

    /// Determines whether the query should be processed or not.
    ///
    /// Returns `Some(QueryPolicyResult)` to process or reject the query, or
    /// `None` to take no action.
    #[allow(unused_variables)]
    fn check_query<'a>(&'a self, query: &'a Filter) -> BoxedFuture<'a, Option<QueryPolicyResult>> {
        Box::pin(async { None })
    }
}

impl<S> PluginsManagerBuilder<S>
where
    S: Clone + Sync + Send,
{
    /// Adds an `any` plugin.
    ///
    /// executing after `all` plugins in order.
    #[inline]
    pub fn add_any<P>(mut self, plugin: P) -> Self
    where
        P: RelayPlugin + 'static,
    {
        self.any_plugins.push(Box::new(plugin));
        self
    }

    /// Adds an `all` plugin.
    ///
    /// executing in order.
    #[inline]
    pub fn add_all<P>(mut self, plugin: P) -> Self
    where
        P: RelayPlugin + 'static,
    {
        self.all_plugins.push(Box::new(plugin));
        self
    }

    /// Add a plugins manager if it's `Some`
    #[inline]
    pub fn add_plugins_manager<P>(mut self, plugins_manager: Option<P>) -> Self
    where
        P: PluginsManagerTrait + 'static,
    {
        if let Some(manager) = plugins_manager {
            self.plugins_managers.push(Box::new(manager));
        }
        self
    }

    /// Add a middleware
    #[inline]
    pub fn add_middleware(mut self, middleware: MiddlewareFn<S>) -> Self {
        self.middlewares.push(middleware);
        self
    }

    /// Add a middlewares state
    pub fn with_middlewares_state(state: S) -> PluginsManagerBuilder<S> {
        PluginsManagerBuilder {
            any_plugins:      Vec::new(),
            all_plugins:      Vec::new(),
            plugins_managers: Vec::new(),
            middlewares:      Vec::new(),
            middleware_state: state,
        }
    }

    /// Build the plugins manager
    #[inline]
    pub fn build(self) -> PluginsManager<S> {
        PluginsManager {
            any_plugins:      Arc::new(self.any_plugins),
            all_plugins:      Arc::new(self.all_plugins),
            plugins_managers: Arc::new(self.plugins_managers),
            middlewares:      Arc::new(self.middlewares),
            middleware_state: self.middleware_state,
        }
    }
}

impl<S> PluginsManager<S>
where
    S: Clone + Send + Sync,
{
    /// Returns the first reject if no plugin accepted the event.
    #[inline(always)]
    async fn check_write_any_plugins(&self, event: &Event) -> Option<WritePolicyResult> {
        let mut first_reject = None;

        for plugin in self.any_plugins.as_ref() {
            match plugin.check_event(event).await {
                Some(WritePolicyResult::Accept) => {
                    // If one plugin accept the event, set the rejection to `None`
                    // and exit the for-loop
                    first_reject = None;
                    break;
                }
                Some(reject) if first_reject.is_none() => first_reject = Some(reject),
                // For subsequent rejections, we don't need to store them
                _ => {}
            }
        }

        // If there is a rejection, there is no plugin accept the event
        first_reject
    }

    /// Returns the first reject if no plugin accepted the query.
    #[inline(always)]
    async fn check_query_any_plugins(&self, query: &Filter) -> Option<QueryPolicyResult> {
        let mut first_reject = None;

        for plugin in self.any_plugins.as_ref() {
            match plugin.check_query(query).await {
                Some(QueryPolicyResult::Accept) => {
                    // If one plugin accept the query, set the rejection to `None`
                    // and exit the for-loop
                    first_reject = None;
                    break;
                }
                Some(reject) if first_reject.is_none() => first_reject = Some(reject),
                // For subsequent rejections, we don't need to store them
                _ => {}
            }
        }

        // If there is a rejection, there is no plugin accept the query
        first_reject
    }
}

impl<S> WritePolicy for PluginsManager<S>
where
    S: Clone + Send + Sync,
{
    fn admit_event<'a>(
        &'a self,
        event: &'a Event,
        addr: &'a SocketAddr,
    ) -> BoxedFuture<'a, WritePolicyResult> {
        Box::pin(async {
            // All of the `all-plugins` must accept the event
            for plugin in self.all_plugins.as_ref() {
                // Return the first reject
                if let Some(reject @ WritePolicyResult::Reject { .. }) =
                    plugin.check_event(event).await
                {
                    return reject;
                }
            }

            if let Some(reject_result) = self.check_write_any_plugins(event).await {
                return reject_result;
            }

            for manager in self.plugins_managers.as_ref() {
                if let reject @ WritePolicyResult::Reject { .. } =
                    manager.admit_event(event, addr).await
                {
                    return reject;
                }
            }

            // Run the middlewares
            futures::future::join_all(
                self.middlewares
                    .as_ref()
                    .iter()
                    .map(|m_fn| m_fn(Either::Right(event), self.middleware_state.clone())),
            )
            .await;

            WritePolicyResult::Accept
        })
    }
}

impl<S> QueryPolicy for PluginsManager<S>
where
    S: Clone + Send + Sync,
{
    fn admit_query<'a>(
        &'a self,
        query: &'a Filter,
        addr: &'a SocketAddr,
    ) -> BoxedFuture<'a, QueryPolicyResult> {
        Box::pin(async {
            // All of the `all-plugins` must accept the query
            for plugin in self.all_plugins.as_ref() {
                // Return the first reject
                if let Some(reject @ QueryPolicyResult::Reject { .. }) =
                    plugin.check_query(query).await
                {
                    return reject;
                }
            }

            if let Some(reject) = self.check_query_any_plugins(query).await {
                return reject;
            }

            for manager in self.plugins_managers.as_ref() {
                if let reject @ QueryPolicyResult::Reject { .. } =
                    manager.admit_query(query, addr).await
                {
                    return reject;
                }
            }

            // Run the middlewares
            futures::future::join_all(
                self.middlewares
                    .as_ref()
                    .iter()
                    .map(|m_fn| m_fn(Either::Left(query), self.middleware_state.clone())),
            )
            .await;

            QueryPolicyResult::Accept
        })
    }
}

impl<S> fmt::Debug for PluginsManager<S>
where
    S: Clone + Send + Sync,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PluginsManager")
            .field("any_plugins", &self.any_plugins.len())
            .field("all_plugins", &self.all_plugins.len())
            .finish()
    }
}

impl Default for PluginsManagerBuilder<()> {
    fn default() -> Self {
        Self {
            any_plugins:      Default::default(),
            all_plugins:      Default::default(),
            plugins_managers: Default::default(),
            middlewares:      Default::default(),
            middleware_state: Default::default(),
        }
    }
}
