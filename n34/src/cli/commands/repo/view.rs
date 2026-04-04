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

use std::fmt;

use clap::Args;
use nostr::nips::nip19::ToBech32;

use crate::{
    cli::{
        CliOptions,
        CommandRunner,
        traits::{OptionNaddrOrSetVecExt, RelayOrSetVecExt},
        types::NaddrOrSet,
    },
    error::N34Result,
    nostr_utils::{NostrClient, traits::NaddrsUtils, utils},
};

/// Arguments for the `repo view` command
#[derive(Args, Debug)]
pub struct ViewArgs {
    /// Repository address in `naddr` format (`naddr1...`), NIP-05 format
    /// (`4rs.nl/n34` or `_@4rs.nl/n34`), or a set name like `kernel`.
    ///
    /// If omitted, looks for a `nostr-address` file.
    #[arg(value_name = "NADDR-NIP05-OR-SET")]
    naddrs: Option<Vec<NaddrOrSet>>,
}

impl CommandRunner for ViewArgs {
    const NEED_SIGNER: bool = false;

    async fn run(self, options: CliOptions) -> N34Result<()> {
        let naddrs = utils::check_empty_naddrs(utils::naddrs_or_file(
            self.naddrs.flat_naddrs(&options.config.sets)?,
            &utils::nostr_address_path()?,
        )?)?;
        let relays = options.relays.clone().flat_relays(&options.config.sets)?;
        let client = NostrClient::init(&options, &relays).await;
        client.add_relays(&naddrs.extract_relays()).await;

        let repos = client.fetch_repos(&naddrs.into_coordinates()).await?;
        let mut repos_details: Vec<String> = Vec::new();

        for repo in repos {
            let mut repo_details = format!("ID: {}", repo.id);

            if let Some(name) = repo.name {
                repo_details.push_str(&format!("\nName: {name}"));
            }
            if let Some(desc) = repo.description {
                repo_details.push_str(&format!("\nDescription: {desc}"));
            }
            if !repo.web.is_empty() {
                repo_details.push_str(&format!("\nWebpages:\n{}", format_list(repo.web)));
            }
            if !repo.clone.is_empty() {
                repo_details.push_str(&format!("\nClone urls:\n{}", format_list(repo.clone)));
            }
            if !repo.relays.is_empty() {
                repo_details.push_str(&format!("\nRelays:\n{}", format_list(repo.relays)));
            }
            if let Some(euc) = repo.euc {
                repo_details.push_str(&format!("\nEarliest unique commit: {euc}"));
            }
            if !repo.maintainers.is_empty() {
                repo_details.push_str(&format!(
                    "\nMaintainers:\n{}",
                    format_list(
                        repo.maintainers
                            .iter()
                            .map(|p| p.to_bech32().expect("Infallible"))
                    )
                ));
            }
            repos_details.push(repo_details);
        }

        println!("{}", repos_details.join("\n----------\n"));
        Ok(())
    }
}

/// Format a vector to print it
fn format_list<I, T>(iterator: I) -> String
where
    I: IntoIterator<Item = T>,
    T: fmt::Display,
{
    iterator
        .into_iter()
        .map(|t| format!(" - {t}"))
        .collect::<Vec<String>>()
        .join("\n")
}
