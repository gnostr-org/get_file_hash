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

use clap::Args;
use nostr::types::RelayUrl;

use crate::{
    cli::{CliOptions, traits::CommandRunner},
    error::N34Result,
};

#[derive(Args, Debug)]
pub struct RelaysArgs {
    /// List of relay URLs to append to fallback relays. If empty, removes all
    /// fallback relays.
    relays:          Vec<RelayUrl>,
    /// Replace existing fallback relays instead of appending new ones.
    #[arg(long = "override")]
    override_relays: bool,
}

impl CommandRunner for RelaysArgs {
    const NEED_SIGNER: bool = false;

    async fn run(self, mut options: CliOptions) -> N34Result<()> {
        if self.relays.is_empty() {
            options.config.fallback_relays = None;
        } else if self.override_relays {
            options.config.fallback_relays = Some(self.relays);
        } else {
            let mut relays = options.config.fallback_relays.clone().unwrap_or_default();
            relays.extend(self.relays);
            relays.sort_unstable();
            relays.dedup();
            options.config.fallback_relays = Some(relays);
        }

        options.config.dump()
    }
}
