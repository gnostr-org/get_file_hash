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

use std::net::SocketAddr;

use clap::{ArgGroup, Args};

use crate::{
    cli::{CliOptions, options_state::DEFAULT_NIP07_PROXY_ADDR, traits::CommandRunner},
    error::N34Result,
};

#[derive(Args, Debug)]
#[clap(
    group(
        ArgGroup::new("options")
            .required(true)
    )
)]
pub struct Nip07Args {
    /// Enable NIP-07 as the default signer.
    #[arg(long, group = "options")]
    enable:  bool,
    /// Disable NIP-07 as the default signer.
    #[arg(long, group = "options", group = "disable_options")]
    disable: bool,
    /// Set the default `ip:port` for the browser signer proxy.
    #[arg(long, group = "disable_options")]
    addr:    Option<SocketAddr>,
}

impl CommandRunner for Nip07Args {
    const NEED_SIGNER: bool = false;

    async fn run(self, mut options: CliOptions) -> N34Result<()> {
        if self.enable {
            let addr = self.addr.unwrap_or(DEFAULT_NIP07_PROXY_ADDR);
            options.config.nip07 = Some(addr)
        } else {
            options.config.nip07 = None
        }

        options.config.dump()
    }
}
