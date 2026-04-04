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

use crate::{
    cli::{
        CliOptions,
        traits::{CommandRunner, MutRepoRelaySetsExt, NaddrOrSetVecExt, RelayOrSetVecExt},
        types::{NaddrOrSet, RelayOrSet},
    },
    error::N34Result,
};

#[derive(Args, Debug)]
pub struct RemoveArgs {
    /// Set name to delete
    name:   String,
    /// Specific relay to remove it from the set, either as URL or set name to
    /// extract its relays. [aliases: `--sr`]
    #[arg(long = "set-relay", alias("sr"))]
    relays: Vec<RelayOrSet>,

    /// Repository address in `naddr` format (`naddr1...`), NIP-05 format
    /// (`4rs.nl/n34` or `_@4rs.nl/n34`), or a set name like `kernel`.
    #[arg(value_name = "NADDR-NIP05-OR-SET", long = "repo")]
    naddrs: Vec<NaddrOrSet>,
}

impl CommandRunner for RemoveArgs {
    const NEED_SIGNER: bool = false;

    async fn run(self, mut options: CliOptions) -> N34Result<()> {
        let naddrs = self.naddrs.flat_naddrs(&options.config.sets)?;
        let relays = self.relays.flat_relays(&options.config.sets)?;

        if relays.is_empty() && naddrs.is_empty() {
            options.config.sets.remove_set(self.name)?;
        } else {
            if !relays.is_empty() {
                options
                    .config
                    .sets
                    .remove_relays(&self.name, relays.into_iter())?;
            }
            if !naddrs.is_empty() {
                options
                    .config
                    .sets
                    .remove_naddrs(self.name, naddrs.into_iter())?;
            }
        }

        options.config.dump()
    }
}
