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

use std::num::NonZeroUsize;

use clap::Args;

use crate::{
    cli::{CliOptions, common_commands, traits::CommandRunner, types::NaddrOrSet},
    error::N34Result,
};

#[derive(Debug, Args)]
pub struct ListArgs {
    /// Repository address in `naddr` format (`naddr1...`), NIP-05 format
    /// (`4rs.nl/n34` or `_@4rs.nl/n34`), or a set name like `kernel`.
    ///
    /// If omitted, looks for a `nostr-address` file.
    #[arg(value_name = "NADDR-NIP05-OR-SET")]
    naddrs: Option<Vec<NaddrOrSet>>,
    /// Maximum number of issues to list
    #[arg(long, default_value = "15")]
    limit:  NonZeroUsize,
}

impl CommandRunner for ListArgs {
    const NEED_SIGNER: bool = false;

    async fn run(self, options: CliOptions) -> N34Result<()> {
        common_commands::list_patches_and_issues(options, self.naddrs, false, self.limit.into())
            .await
    }
}
