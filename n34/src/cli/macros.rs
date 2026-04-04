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

use super::traits::CommandRunner;


/// Returns whether the command runner type `T` requires relays.
pub fn get_relays_state<T: CommandRunner>(_v: &T) -> bool {
    T::NEED_RELAYS
}

/// Returns whether the command runner type `T` requires a signer.
pub fn get_signer_state<T: CommandRunner>(_v: &T) -> bool {
    T::NEED_SIGNER
}

/// Executes a command with required setup checks. The first parameter is the
/// command to match on (often `self`), followed by options. Optional
/// subcommands come next, and commands with arguments (after `&`) are listed
/// last.
#[macro_export]
macro_rules! run_command {
    ($command:ident, $options:ident, $($subcommands:ident)* & $($commands:ident)*) => {
        match $command {
            $(
                Self::$subcommands { subcommands } => subcommands.run($options).await,
            )*
            $(
                Self::$commands ( args ) => {
                    if $crate::cli::macros::get_relays_state(&args) {
                        $options.ensure_relays()?;
                    }
                    if $crate::cli::macros::get_signer_state(&args) {
                        $options.ensure_signer()?;
                    }
                    args.run($options).await
                },
            )*
        }
    };
}
