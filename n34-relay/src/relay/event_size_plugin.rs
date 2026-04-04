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

use nostr::{event::Event, message::MachineReadablePrefix, util::BoxedFuture};
use nostr_relay_builder::builder::WritePolicyResult;

use crate::relay::plugins_manager::RelayPlugin;

/// A plugin to check the event size.
#[derive(Debug)]
pub struct EventSizePlugin(pub usize);

impl RelayPlugin for EventSizePlugin {
    fn check_event<'a>(&'a self, event: &'a Event) -> BoxedFuture<'a, Option<WritePolicyResult>> {
        Box::pin(async move {
            let event_size = event
                .tags
                .iter()
                .flat_map(|t| t.as_slice())
                .fold(0, |acc: usize, tag_content| {
                    acc.saturating_add(tag_content.len())
                })
                .saturating_add(event.content.len());

            if event_size > self.0 {
                return Some(WritePolicyResult::reject(
                    MachineReadablePrefix::Blocked,
                    format!(
                        "event size {event_size} is larger than maximum allowed {}",
                        self.0
                    ),
                ));
            }

            None
        })
    }
}
