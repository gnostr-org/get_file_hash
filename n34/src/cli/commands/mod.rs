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

/// `config` subcommands
pub mod config;
/// `issue` subcommands
pub mod issue;
/// `patch` subcommands
pub mod patch;
/// `reply` command
pub mod reply;
/// `repo` subcommands
pub mod repo;
/// `sets` subcommands
pub mod sets;

use std::fmt;
use std::sync::Arc;
use std::time::Duration;

use clap::{Args, Parser};
use nostr::key::{Keys, SecretKey};
use nostr::nips::nip46::NostrConnectURI;
use nostr::signer::{IntoNostrSigner, NostrSigner};
use nostr_connect::client::NostrConnect;

use self::config::ConfigSubcommands;
use self::issue::IssueSubcommands;
use self::patch::PatchSubcommands;
use self::reply::ReplyArgs;
use self::repo::RepoSubcommands;
use self::sets::SetsSubcommands;
use super::CliConfig;
use super::options_state::OptionsState;
use super::types::RelayOrSet;
use super::{parsers, traits::CommandRunner};
use crate::cli::Cli;
use crate::cli::types::EchoAuthUrl;
use crate::error::{N34Error, N34Result};

/// Default path used when no path is provided via command line arguments.
///
/// This is a workaround since Clap doesn't support lazy evaluation of defaults.
pub const DEFAULT_FALLBACK_PATH: &str = "I_DO_NOT_KNOW_WHY_CLAP_DO_NOT_SUPPORT_LAZY_DEFAULT";

/// How long to wait for bunker response (3 minutes).
const BUNKER_TIMEOUT: Duration = Duration::from_secs(60 * 3);

/// The command-line interface options
#[derive(Args)]
pub struct CliOptions {
    /// Your Nostr secret key
    #[arg(short, long, group = "signer")]
    pub secret_key: Option<SecretKey>,
    /// NIP-46 bunker url used for signing events
    #[arg(short, long, group = "signer", value_parser = parsers::parse_bunker_url)]
    pub bunker_url: Option<NostrConnectURI>,
    /// Enables signing events using the browser's NIP-07 extension. Listens on
    /// `127.0.0.1:51034`.
    #[arg(short = '7', long, group = "signer")]
    pub nip07:      bool,
    /// Fallbacks relay to write and read from it. Multiple relays can be
    /// passed.
    #[arg(short, long)]
    pub relays:     Vec<RelayOrSet>,
    /// Proof of Work difficulty when creatring events
    #[arg(long, value_name = "DIFFICULTY")]
    pub pow:        Option<u8>,
    /// Config path [default: `$XDG_CONFIG_HOME` or `$HOME/.config`]
    #[arg(long, value_name = "PATH", default_value = DEFAULT_FALLBACK_PATH,
         hide_default_value = true, value_parser = parsers::parse_config_path
     )]
    pub config:     CliConfig,
    /// The state of options. Some values that are used by them but should not
    /// be entered via the CLI
    #[arg(skip)]
    pub state:      OptionsState,
}

/// N34 commands
#[derive(Parser, Debug)]
pub enum Commands {
    /// Manage repositories and relays sets
    Sets {
        #[command(subcommand)]
        subcommands: SetsSubcommands,
    },
    /// Manage repositories
    Repo {
        #[command(subcommand)]
        subcommands: RepoSubcommands,
    },
    /// Manage issues
    Issue {
        #[command(subcommand)]
        subcommands: IssueSubcommands,
    },
    /// Manage patches
    Patch {
        #[command(subcommand)]
        subcommands: PatchSubcommands,
    },
    /// Manage configuration
    Config {
        #[command(subcommand)]
        subcommands: ConfigSubcommands,
    },
    /// Reply to issues and patches.
    Reply(ReplyArgs),
}


impl CliOptions {
    /// Returns the signer
    pub async fn signer(&self) -> N34Result<Option<Arc<dyn NostrSigner + 'static>>> {
        if self.nip07 {
            self.state.browser_signer_proxy.start().await?;

            println!(
                "Browser signer proxy started at: {}",
                self.state.browser_signer_proxy.url()
            );

            // FIXME: Use `BrowserSignerProxy::is_session_active` after it release
            // nostr@0.44.0
            tokio::time::sleep(Duration::from_secs(10)).await;

            return Ok(Some(
                self.state.browser_signer_proxy.clone().into_nostr_signer(),
            ));
        }

        if let Some(sk) = &self.secret_key {
            return Ok(Some(Keys::new(sk.clone()).into_nostr_signer()));
        }

        if let Some(ref bunker_url) = self.bunker_url {
            let mut nostrconnect = NostrConnect::new(
                bunker_url.clone(),
                Cli::n34_keypair()?,
                BUNKER_TIMEOUT,
                None,
            )
            .expect("It's a bunker url and not a client");

            nostrconnect.auth_url_handler(EchoAuthUrl);
            return Ok(Some(nostrconnect.into_nostr_signer()));
        }
        Ok(None)
    }

    /// Returns an error if there are no relays.
    pub fn ensure_relays(&self) -> N34Result<()> {
        if self.relays.is_empty() {
            return Err(N34Error::EmptyRelays);
        }
        Ok(())
    }

    /// Returns an error if there are no signers
    pub fn ensure_signer(&self) -> N34Result<()> {
        if !self.config.keyring_secret_key && self.secret_key.is_none() && self.bunker_url.is_none()
        {
            return Err(N34Error::SignerRequired);
        }
        Ok(())
    }
}

impl fmt::Debug for CliOptions {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CliOptions")
            .field("secret_key", &self.secret_key.as_ref().map(|_| "*******"))
            .field("bunker_url", &self.bunker_url.as_ref().map(|_| "*******"))
            .field("relays", &self.relays)
            .field("pow", &self.pow)
            .field("config", &self.config)
            .finish()
    }
}

impl CommandRunner for Commands {
    async fn run(self, options: CliOptions) -> N34Result<()> {
        tracing::trace!("Options: {options:#?}");
        tracing::trace!("Handling: {self:#?}");

        crate::run_command!(self, options, Repo Issue Sets Patch Config & Reply)
    }
}
