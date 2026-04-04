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

/// Commands module
pub mod commands;
/// Common commands used by multiply commands
pub mod common_commands;
/// The CLI config
pub mod config;
/// Default lazy values for CLI arguments
pub mod defaults;
/// Macros for CLI application.
pub mod macros;
/// Represents the state used for CLI options.
pub mod options_state;
/// CLI arguments parsers
pub mod parsers;
/// CLI traits
pub mod traits;
/// Common helper types used throughout the CLI.
pub mod types;


use clap::Parser;
use clap_verbosity_flag::Verbosity;
use nostr::key::Keys;
use nostr::key::SecretKey;
use nostr_browser_signer_proxy::BrowserSignerProxy;
use nostr_browser_signer_proxy::BrowserSignerProxyOptions;
use nostr_keyring::KeyringError;
use nostr_keyring::NostrKeyring;
use types::RelayOrSet;

pub use self::commands::*;
pub use self::config::*;
use self::traits::CommandRunner;
use crate::cli::options_state::BROWSER_SIGNER_PROXY_TIMEOUT;
use crate::error::N34Error;
use crate::error::N34Result;
use crate::nostr_utils::traits::NostrKeyringErrorUtils;

/// Header message, used in the help message
const HEADER: &str = r#"Copyright (C) 2025 Awiteb <a@4rs.nl>
License GNU GPL-3.0-or-later <https://gnu.org/licenses/gpl-3.0.html>
This is free software: you are free to change and redistribute it.
There is NO WARRANTY, to the extent permitted by law.

Git repository: https://git.4rs.nl/awiteb/n34.git"#;

/// Footer message, used in the help message
const FOOTER: &str = r#"Please report bugs to <naddr1qqpkuve5qgsqqqqqq9g9uljgjfcyd6dm4fegk8em2yfz0c3qp3tc6mntkrrhawgrqsqqqauesksc39>."#;


/// Name of the file storing the repository address
pub const NOSTR_ADDRESS_FILE: &str = "nostr-address";

#[derive(Parser, Debug)]
#[command(about, version, before_long_help = HEADER, after_long_help = FOOTER)]
/// A command-line interface for interacting with NIP-34 and other Nostr
/// code-related stuff.
pub struct Cli {
    #[command(flatten)]
    pub options:   commands::CliOptions,
    /// Controls the verbosity level of output
    #[command(flatten)]
    pub verbosity: Verbosity,
    /// The subcommand to execute
    #[command(subcommand)]
    pub command:   commands::Commands,
}


impl Cli {
    /// Keyring service name of n34
    pub const N34_KEYRING_SERVICE_NAME: &str = "n34";
    /// Keyring entry name of the n34 keypair
    pub const N34_KEY_PAIR_ENTRY: &str = "n34_keypair";
    /// Keyring entry name of the user secret key
    pub const USER_KEY_PAIR_ENTRY: &str = "user_keypair";

    /// Executes the command
    pub async fn run(self) -> N34Result<()> {
        self.command.run(self.options).await
    }

    /// Gets the n34 keypair from the keyring or generates and stores a new one
    /// if none exists.
    pub fn n34_keypair() -> N34Result<Keys> {
        let keyring = NostrKeyring::new(Self::N34_KEYRING_SERVICE_NAME);

        match keyring.get(Self::N34_KEY_PAIR_ENTRY) {
            Ok(keys) => Ok(keys),
            Err(nostr_keyring::Error::Keyring(KeyringError::NoEntry)) => {
                let new_keys = Keys::generate();
                keyring.set(Self::N34_KEY_PAIR_ENTRY, &new_keys)?;
                Ok(new_keys)
            }
            Err(err) => Err(N34Error::Keyring(err)),
        }
    }

    /// Retrieves the user's keypair from the keyring. If no key exists and one
    /// is provided, stores and returns it. If no key exists and none is
    /// provided, returns an error.
    pub fn user_keypair(secret_key: Option<SecretKey>) -> N34Result<Keys> {
        let keyring = NostrKeyring::new(Self::N34_KEYRING_SERVICE_NAME);
        let keyring_key = keyring.get(Self::USER_KEY_PAIR_ENTRY);

        if let Err(ref err) = keyring_key
            && err.is_keyring_no_entry()
            && let Some(secret_key) = secret_key
        {
            let keypair = Keys::new(secret_key);
            keyring.set(Self::USER_KEY_PAIR_ENTRY, &keypair)?;
            return Ok(keypair);
        }

        keyring_key.map_err(|err| {
            if err.is_keyring_no_entry() {
                N34Error::SecretKeyKeyringWithoutEntry
            } else {
                N34Error::Keyring(err)
            }
        })
    }
}

/// Processes the CLI configuration by applying fallback values from config if
/// needed. Returns the processed Cli configuration if successful.
pub fn post_cli(mut cli: Cli) -> N34Result<Cli> {
    cli.options.pow = cli.options.pow.or(cli.options.config.pow);

    if cli.options.relays.is_empty()
        && let Some(relays) = &cli.options.config.fallback_relays
    {
        cli.options.relays = relays.iter().cloned().map(RelayOrSet::Relay).collect();
    }

    // Automatically sets the signer based on the configuration if no signer
    // is provided.
    if !cli.options.nip07
        && cli.options.bunker_url.is_none()
        && (cli.options.secret_key.is_none() || cli.options.config.keyring_secret_key)
    {
        if let Some(addr) = cli.options.config.nip07 {
            cli.options.nip07 = true;
            cli.options.state.browser_signer_proxy = BrowserSignerProxy::new(
                BrowserSignerProxyOptions::default()
                    .timeout(BROWSER_SIGNER_PROXY_TIMEOUT)
                    .ip_addr(addr.ip())
                    .port(addr.port()),
            );
        } else if let Some(bunker_url) = &cli.options.config.bunker_url {
            cli.options.bunker_url = Some(bunker_url.clone());
        } else if cli.options.config.keyring_secret_key {
            cli.options.secret_key = Some(
                Cli::user_keypair(cli.options.secret_key)?
                    .secret_key()
                    .clone(),
            );
        }
    }

    Ok(cli)
}
