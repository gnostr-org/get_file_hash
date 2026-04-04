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
use nostr::nips::nip19::ToBech32;

use crate::{
    cli::{
        CliOptions,
        RepoRelaySet,
        traits::{CommandRunner, RepoRelaySetsExt},
    },
    error::N34Result,
};

#[derive(Args, Debug)]
pub struct ShowArgs {
    /// Name of the set to display. If not provided, lists all available sets.
    name: Option<String>,
}

impl CommandRunner for ShowArgs {
    const NEED_SIGNER: bool = false;

    async fn run(self, options: CliOptions) -> N34Result<()> {
        if let Some(name) = self.name {
            println!(
                "{}",
                format_set(options.config.sets.as_slice().get_set(&name)?)
            );
        } else {
            println!(
                "{}",
                options
                    .config
                    .sets
                    .iter()
                    .map(format_set)
                    .collect::<Vec<_>>()
                    .join("\n----------\n")
            );
        }

        Ok(())
    }
}

/// Format a set to view it to the user
fn format_set(set: &RepoRelaySet) -> String {
    let naddrs = if set.naddrs.is_empty() {
        "Nothing".to_owned()
    } else {
        format!(
            "\n- {}",
            set.naddrs
                .iter()
                .map(|naddr| naddr.to_bech32().expect("We did decoded before"))
                .collect::<Vec<_>>()
                .join("\n- ")
        )
    };
    let relays = if set.relays.is_empty() {
        "Nothing".to_owned()
    } else {
        format!(
            "\n- {}",
            set.relays
                .iter()
                .map(|relay| relay.to_string())
                .collect::<Vec<_>>()
                .join("\n- ")
        )
    };

    format!(
        "Name: {}\nّّRepositories: {naddrs}\nRelays: {relays}",
        set.name
    )
}
