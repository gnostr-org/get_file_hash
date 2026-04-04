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

use clap::{ArgGroup, Args};

use crate::{
    cli::{Cli, CliOptions, traits::CommandRunner},
    error::N34Result,
};

#[derive(Args, Debug)]
#[clap(
    group(
        ArgGroup::new("options")
            .required(true)
    )
)]
pub struct KeyringArgs {
    /// Turns on secret key keyring. Requires entering the key once when
    /// enabled.
    #[arg(long, group = "options")]
    enable:  bool,
    /// Turns off secret key keyring. Removes any existing key and prevents
    /// storing new ones.
    #[arg(long, group = "options")]
    disable: bool,
    /// Deletes current key and stores the next provided key.
    #[arg(long, group = "options")]
    reset:   bool,
}

impl CommandRunner for KeyringArgs {
    const NEED_SIGNER: bool = false;

    async fn run(self, mut options: CliOptions) -> N34Result<()> {
        let keyring = nostr_keyring::NostrKeyring::new(Cli::N34_KEYRING_SERVICE_NAME);

        if self.enable {
            options.config.keyring_secret_key = true;
        } else if self.disable {
            options.config.keyring_secret_key = false;
        }

        if self.reset || self.disable {
            let _ = keyring.delete(Cli::USER_KEY_PAIR_ENTRY);
        }

        options.config.dump()
    }
}
