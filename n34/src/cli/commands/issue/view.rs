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
use nostr::{event::Kind, filter::Filter};

use crate::{
    cli::{
        CliOptions,
        traits::{CommandRunner, OptionNaddrOrSetVecExt, RelayOrSetVecExt},
        types::{NaddrOrSet, NostrEvent},
    },
    error::{N34Error, N34Result},
    nostr_utils::{
        NostrClient,
        traits::{GitIssueUtils, NaddrsUtils, ReposUtils},
        utils,
    },
};

#[derive(Debug, Args)]
pub struct ViewArgs {
    /// Repository address in `naddr` format (`naddr1...`), NIP-05 format
    /// (`4rs.nl/n34` or `_@4rs.nl/n34`), or a set name like `kernel`.
    ///
    /// If omitted, looks for a `nostr-address` file.
    #[arg(value_name = "NADDR-NIP05-OR-SET", long = "repo")]
    naddrs:   Option<Vec<NaddrOrSet>>,
    /// The issue id to view it
    issue_id: NostrEvent,
}

impl CommandRunner for ViewArgs {
    const NEED_SIGNER: bool = false;

    async fn run(self, options: CliOptions) -> N34Result<()> {
        let naddrs = utils::naddrs_or_file(
            self.naddrs.flat_naddrs(&options.config.sets)?,
            &utils::nostr_address_path()?,
        )?;
        let relays = options.relays.clone().flat_relays(&options.config.sets)?;
        let client = NostrClient::init(&options, &relays).await;

        client.add_relays(&naddrs.extract_relays()).await;
        client.add_relays(&self.issue_id.relays).await;
        client
            .add_relays(
                &client
                    .fetch_repos(&naddrs.into_coordinates())
                    .await?
                    .extract_relays(),
            )
            .await;

        let issue = client
            .fetch_event(
                Filter::new()
                    .id(self.issue_id.event_id)
                    .kind(Kind::GitIssue),
            )
            .await?
            .ok_or(N34Error::CanNotFoundIssue)?;

        let issue_subject = utils::smart_wrap(issue.extract_issue_subject(), 70);
        let issue_author = client.get_username(issue.pubkey).await;
        let mut issue_labels = utils::smart_wrap(&issue.extract_issue_labels(), 70);

        if issue_labels.is_empty() {
            issue_labels = "\n".to_owned();
        } else {
            issue_labels = format!("{issue_labels}\n\n")
        }

        println!(
            "{issue_subject} - [by {issue_author}]\n{issue_labels}{}",
            utils::smart_wrap(&issue.content, 80)
        );
        Ok(())
    }
}
