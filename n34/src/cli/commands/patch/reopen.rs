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

use super::PatchStatus;
use crate::{
    cli::{
        CliOptions,
        traits::CommandRunner,
        types::{NaddrOrSet, NostrEvent},
    },
    error::{N34Error, N34Result},
};

#[derive(Debug, Args)]
pub struct ReopenArgs {
    /// Repository address in `naddr` format (`naddr1...`), NIP-05 format
    /// (`4rs.nl/n34` or `_@4rs.nl/n34`), or a set name like `kernel`.
    ///
    /// If omitted, looks for a `nostr-address` file.
    #[arg(value_name = "NADDR-NIP05-OR-SET", long = "repo")]
    naddrs:   Option<Vec<NaddrOrSet>>,
    /// The closed/drafted patch id to reopen it. Must be orignal root patch
    patch_id: NostrEvent,
}

impl CommandRunner for ReopenArgs {
    async fn run(self, options: CliOptions) -> N34Result<()> {
        crate::cli::common_commands::patch_status_command(
            options,
            self.patch_id,
            self.naddrs,
            PatchStatus::Open,
            None,
            Vec::new(),
            |patch_status| {
                if patch_status.is_open() {
                    return Err(N34Error::InvalidStatus(
                        "You can't open an already open patch".to_owned(),
                    ));
                }

                if patch_status.is_merged_or_applied() {
                    return Err(N34Error::InvalidStatus(
                        "You can't open a merged/applied patch".to_owned(),
                    ));
                }

                Ok(())
            },
        )
        .await
    }
}
