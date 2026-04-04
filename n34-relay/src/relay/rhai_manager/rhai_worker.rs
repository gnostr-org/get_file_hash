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

use either::Either;
use nostr::{event::Event, filter::Filter};

use super::{rhai_caller::WorkerRx, rhai_runner::RhaiPluginsRunner};

/// Worker that receives and executes Rhai script calls.
pub struct RhaiWorker;

impl RhaiWorker {
    /// Spawns a new worker that processes incoming events and queries.
    ///
    /// The worker continuously receives messages through `rx` and processes
    /// them using the provided `runner`. Events and queries are handled as
    /// they arrive.
    ///
    /// # Safety
    /// The dereference is safe because the caller waits for the response and
    /// won't drop the reference while it's being used.
    pub fn create(rx: WorkerRx, runner: Arc<RhaiPluginsRunner>) {
        tokio::spawn(async move {
            while let Ok((event_or_filter, response_tx)) = rx.recv_async().await {
                let res = match event_or_filter {
                    Either::Left(event_addr) => {
                        runner.admit_event(unsafe { &*(event_addr as *const Event) })
                    }
                    Either::Right(filter_addr) => {
                        runner.admit_query(unsafe { &*(filter_addr as *const Filter) })
                    }
                };
                _ = response_tx.send(res);
            }
        });
    }
}
