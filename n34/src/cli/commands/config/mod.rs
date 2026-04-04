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

/// `config bunker` subcommand
mod bunker;
/// `config keyring` subcommand
mod keyring;
/// `config nip07` subcommand
mod nip07;
/// `config pow` subcommand
mod pow;
/// `config relays` subcommand
mod relays;

use clap::Subcommand;

use self::bunker::BunkerArgs;
use self::keyring::KeyringArgs;
use self::nip07::Nip07Args;
use self::pow::PowArgs;
use self::relays::RelaysArgs;
use super::CliOptions;
use crate::{cli::traits::CommandRunner, error::N34Result};


#[derive(Subcommand, Debug)]
pub enum ConfigSubcommands {
    /// Sets the default PoW difficulty (0 if not specified)
    Pow(PowArgs),
    /// Sets the default fallback relays if none provided. Use this relays for
    /// read and write.
    Relays(RelaysArgs),
    /// Sets a URL of NIP-46 bunker server used for signing events.
    Bunker(BunkerArgs),
    /// Managing the secret key keyring, including enabling, disabling, or
    /// resetting it.
    Keyring(KeyringArgs),
    /// Controls the NIP-07 browser signer proxy, turning it on or off, and
    /// configures the `ip:port` address.
    Nip07(Nip07Args),
}

impl CommandRunner for ConfigSubcommands {
    async fn run(self, options: CliOptions) -> N34Result<()> {
        crate::run_command!(self, options, & Pow Relays Bunker Keyring Nip07)
    }
}
