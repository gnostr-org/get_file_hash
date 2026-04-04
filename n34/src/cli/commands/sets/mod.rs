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

/// `sets new` command
mod new;
/// `sets remove` command
mod remove;
/// `sets show` commands
mod show;
/// `sets update` command
mod update;

use clap::Subcommand;

use self::new::NewArgs;
use self::remove::RemoveArgs;
use self::show::ShowArgs;
use self::update::UpdateArgs;
use super::{CliOptions, CommandRunner};
use crate::error::N34Result;

#[derive(Subcommand, Debug)]
pub enum SetsSubcommands {
    /// Remove a set, or specific repos and relays within it
    Remove(RemoveArgs),
    /// Create a new set
    New(NewArgs),
    /// Modify an existing set
    Update(UpdateArgs),
    /// Show a single set or all the stored sets
    Show(ShowArgs),
}


impl CommandRunner for SetsSubcommands {
    async fn run(self, options: CliOptions) -> N34Result<()> {
        crate::run_command!(self, options, & Remove New Update Show)
    }
}
