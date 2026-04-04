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

use std::{path::PathBuf, sync::Arc};

use nostr::{key::PublicKey, nips::nip19::ToBech32};
use nostr_database::NostrDatabase;
use nostr_relay_builder::LocalRelay;

use crate::relay_config::RelayConfig;

/// Router state.
pub struct RouterState {
    /// Configuration settings for the relay.
    pub config:   Arc<RelayConfig>,
    /// The local relay instance being managed.
    pub relay:    Arc<LocalRelay>,
    /// The relay database
    pub database: Arc<dyn NostrDatabase>,
}

impl RouterState {
    /// Creates a new router state.
    pub fn new(
        config: Arc<RelayConfig>,
        relay: Arc<LocalRelay>,
        database: Arc<dyn NostrDatabase>,
    ) -> Self {
        Self {
            config,
            relay,
            database,
        }
    }

    /// Returns the repo path if it's exists
    pub fn repo_path(&self, npub: &PublicKey, repo_name: &str) -> Option<PathBuf> {
        let path = self
            .config
            .grasp
            .repos_path
            .join(npub.to_bech32().expect("Infallible"))
            .join(repo_name)
            .with_extension("git");

        path.exists().then_some(path)
    }
}
