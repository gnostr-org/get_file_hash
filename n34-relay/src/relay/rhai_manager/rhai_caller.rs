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

use either::Either;
use nostr::{event::Event, filter::Filter, message::MachineReadablePrefix, util::BoxedFuture};
use nostr_relay_builder::builder::{
    QueryPolicy,
    QueryPolicyResult,
    WritePolicy,
    WritePolicyResult,
};
use tokio::sync::oneshot;

use crate::relay::plugins_manager::PluginsManagerTrait;

/// Represents an event or a filter address. The worker checks and executes
/// either `write` or `query` plugins based on this.
type EventOrFilter = Either<usize, usize>;

/// Response channel, where the response will be sent
type ResponseChannel = oneshot::Sender<Result<(), String>>;

/// Channel sender for passing work items (EventOrFilter + ResponseChannel) to
/// workers.
pub type WorkerTx = flume::Sender<(EventOrFilter, ResponseChannel)>;

/// Channel receiver for workers to get work items (EventOrFilter +
/// ResponseChannel).
pub type WorkerRx = flume::Receiver<(EventOrFilter, ResponseChannel)>;

/// Call a rhai plugin function.
///
/// The call is done by the provaided channel, the channel is mpmc, caller will
/// send the call body (event or filter) and a channel to return the response
#[derive(Debug, Clone)]
pub struct PluginCaller(pub WorkerTx);

impl PluginCaller {
    async fn call(&self, param: EventOrFilter) -> Result<(), String> {
        let (response_tx, response_rx) = oneshot::channel();
        _ = self.0.send((param, response_tx));

        match response_rx.await {
            Ok(rhai_result) => rhai_result,
            Err(_) => {
                // This should not happen
                tracing::error!("Rhai worker dropped the response sender before sending");
                Ok(())
            }
        }
    }

    /// Call write plugins with the given event.
    ///
    /// Returns Err(reject_msg) if the event rejected
    pub async fn call_write(&self, event: &Event) -> Result<(), String> {
        // SAFETY: This function is safe because it holds a reference and ensures the
        // worker responds before proceeding.
        self.call(Either::Left(event as *const Event as usize))
            .await
    }

    /// Call query plugins with the given filter
    ///
    /// Returns Err(reject_msg) if the query rejected
    pub async fn call_query(&self, filter: &Filter) -> Result<(), String> {
        // SAFETY: This function is safe because it holds a reference and ensures the
        // worker responds before proceeding.
        self.call(Either::Right(filter as *const Filter as usize))
            .await
    }
}

impl WritePolicy for PluginCaller {
    fn admit_event<'a>(
        &'a self,
        event: &'a Event,
        _: &'a std::net::SocketAddr,
    ) -> BoxedFuture<'a, WritePolicyResult> {
        Box::pin(async move {
            if let Err(reject_msg) = self.call_write(event).await {
                return WritePolicyResult::reject(MachineReadablePrefix::Blocked, reject_msg);
            }
            WritePolicyResult::Accept
        })
    }
}

impl QueryPolicy for PluginCaller {
    fn admit_query<'a>(
        &'a self,
        query: &'a Filter,
        _: &'a std::net::SocketAddr,
    ) -> BoxedFuture<'a, QueryPolicyResult> {
        Box::pin(async move {
            if let Err(reject_msg) = self.call_query(query).await {
                return QueryPolicyResult::reject(MachineReadablePrefix::Blocked, reject_msg);
            }
            QueryPolicyResult::Accept
        })
    }
}

impl PluginsManagerTrait for PluginCaller {}
