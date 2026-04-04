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
    cli::{CliOptions, traits::CommandRunner},
    error::N34Result,
};

#[derive(Args, Debug)]
pub struct PowArgs {
    /// The new default PoW difficulty
    difficulty: u8,
}

impl CommandRunner for PowArgs {
    const NEED_SIGNER: bool = false;

    async fn run(self, mut options: CliOptions) -> N34Result<()> {
        options.config.pow = Some(self.difficulty);
        options.config.dump()
    }
}
