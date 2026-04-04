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

use std::{fs, io::Write};

use clap::Args;
use futures::future;
use nostr::{event::EventBuilder, key::PublicKey, types::Url};

use crate::{
    cli::{CliOptions, CommandRunner, NOSTR_ADDRESS_FILE, traits::RelayOrSetVecExt},
    error::N34Result,
    nostr_utils::{NostrClient, traits::NewGitRepositoryAnnouncement, utils},
};

/// Header written to new `nostr-address` files. Contains two trailing newline
/// for formatting.
const NOSTR_ADDRESS_FILE_HEADER: &str = r##"# This file contains NIP-19 `naddr` entities for repositories that accept this
# project's issues and patches.
#
# The file acts as a **read-only reference** for retrieving repository relays
# when embedded in an `naddr` and mentions those repositories when opening
# patches or issues. Modifications here will not affect in the relays, as the
# file is **explicitly untracked**. Its goal is to simplify contributions by
# removing the need for manual address entry.
#
# Each entry must start with "naddr". Embedded relays are **strongly recommended**
# to assist client-side discovery.
#
# Empty lines are ignored. Lines starting with "#" are treated as comments.

"##;

/// Arguments for the `repo announce` command
#[derive(Args, Debug)]
pub struct AnnounceArgs {
    /// Unique identifier for the repository in kebab-case.
    #[arg(long = "id")]
    repo_id:      String,
    /// A name for the repository.
    #[arg(short, long)]
    name:         Option<String>,
    /// A description for the repository.
    #[arg(short, long)]
    description:  Option<String>,
    /// Webpage URLs for the repository (if provided by the git server).
    #[arg(short, long)]
    web:          Vec<Url>,
    /// URLs for cloning the repository.
    #[arg(short, long)]
    clone:        Vec<Url>,
    /// Additional maintainers of the repository (besides yourself).
    #[arg(short, long)]
    maintainers:  Vec<PublicKey>,
    /// Labels to categorize the repository. Can be specified multiple times.
    #[arg(short, long)]
    label:        Vec<String>,
    /// Skip kebab-case validation for the repository ID
    #[arg(long)]
    force_id:     bool,
    /// If set, creates a `nostr-address` file to enable automatic address
    /// discovery by n34
    #[arg(long)]
    address_file: bool,
}

impl CommandRunner for AnnounceArgs {
    const NEED_RELAYS: bool = true;

    async fn run(mut self, options: CliOptions) -> N34Result<()> {
        let relays = options.relays.clone().flat_relays(&options.config.sets)?;
        let client = NostrClient::init(&options, &relays).await;
        let user_pubk = client.pubkey().await?;
        let relays_list = client.user_relays_list(user_pubk).await?;
        client
            .add_relays(&utils::add_read_relays(relays_list.as_ref()))
            .await;

        if !self.maintainers.contains(&user_pubk) {
            self.maintainers.insert(0, user_pubk);
        }

        let naddr = utils::repo_naddr(&self.repo_id, user_pubk, &relays)?;
        let event = EventBuilder::new_git_repo(
            self.repo_id,
            self.name.map(utils::str_trim),
            self.description.map(utils::str_trim),
            self.web,
            self.clone,
            relays.clone(),
            self.maintainers.clone(),
            self.label.into_iter().map(utils::str_trim).collect(),
            self.force_id,
        )?
        .dedup_tags()
        .pow(options.pow.unwrap_or_default())
        .build(user_pubk);


        if self.address_file {
            let address_path = std::env::current_dir()?.join(NOSTR_ADDRESS_FILE);
            if !address_path.exists() {
                tracing::info!(
                    "Creating new address file: '{NOSTR_ADDRESS_FILE}' at path '{}' with default \
                     header",
                    address_path.display()
                );
                fs::write(&address_path, NOSTR_ADDRESS_FILE_HEADER)?;
            }

            let mut file = fs::OpenOptions::new().append(true).open(&address_path)?;

            tracing::info!("Appending naddr '{naddr}' to address file: '{NOSTR_ADDRESS_FILE}'");
            file.write_all(format!("{naddr}\n").as_bytes())?;
            tracing::info!("Successfully wrote naddr to address file");
        }

        let write_relays = [
            relays,
            utils::add_write_relays(relays_list.as_ref()),
            // Include read relays for each maintainer (if found)
            future::join_all(
                self.maintainers
                    .iter()
                    .map(|pkey| client.read_relays_from_user(*pkey)),
            )
            .await
            .into_iter()
            .flatten()
            .collect(),
        ]
        .concat();
        let nevent = utils::new_nevent(event.id.expect("There is an id"), &write_relays)?;

        client
            .send_event_to(event, relays_list.as_ref(), &write_relays)
            .await?;

        println!("Event: {nevent}",);
        println!("Repo Address: {naddr}",);

        Ok(())
    }
}
