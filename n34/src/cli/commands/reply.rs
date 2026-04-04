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

use std::fs;

use clap::{ArgGroup, Args};
use futures::future;
use nostr::{
    event::{Event, EventBuilder, Kind},
    filter::Filter,
    nips::nip01::Coordinate,
    types::RelayUrl,
};

use super::{CliOptions, CommandRunner};
use crate::{
    cli::{
        traits::{OptionNaddrOrSetVecExt, RelayOrSetVecExt},
        types::{NaddrOrSet, NostrEvent},
    },
    error::{N34Error, N34Result},
    nostr_utils::{
        NostrClient,
        traits::{NaddrsUtils, ReposUtils},
        utils,
    },
};

/// The max date "9999-01-01 at 00:00 UTC"
const MAX_DATE: i64 = 253370764800;

/// Arguments for the `reply` command
#[derive(Args, Debug)]
#[clap(
    group(
        ArgGroup::new("comment-content")
            .args(["comment", "editor"])
            .required(true)
    ),
    group(
        ArgGroup::new("quote-reply-to")
            .args(["comment", "quote_to"])
    )
)]
pub struct ReplyArgs {
    /// The issue, patch, or comment to reply to
    #[arg(value_name = "nevent1-or-note1")]
    to:       NostrEvent,
    /// Quote the replied-to event in the editor
    #[arg(long)]
    quote_to: bool,
    /// Repository address in `naddr` format (`naddr1...`), NIP-05 format
    /// (`4rs.nl/n34` or `_@4rs.nl/n34`), or a set name like `kernel`.
    ///
    /// If omitted, looks for a `nostr-address` file.
    #[arg(value_name = "NADDR-NIP05-OR-SET", long = "repo")]
    naddrs:   Option<Vec<NaddrOrSet>>,
    /// The comment (cannot be used with --editor)
    #[arg(short, long)]
    comment:  Option<String>,
    /// Open editor to write comment (cannot be used with --content)
    #[arg(short, long)]
    editor:   bool,
}

impl CommandRunner for ReplyArgs {
    async fn run(self, options: CliOptions) -> N34Result<()> {
        let nostr_address_path = utils::nostr_address_path()?;
        let relays = options.relays.clone().flat_relays(&options.config.sets)?;
        let client = NostrClient::init(&options, &relays).await;
        let user_pubk = client.pubkey().await?;
        let repo_naddrs = if let Some(naddrs) = self.naddrs.flat_naddrs(&options.config.sets)? {
            client.add_relays(&naddrs.extract_relays()).await;
            Some(naddrs)
        } else if fs::exists(&nostr_address_path).is_ok() {
            let naddrs = utils::naddrs_or_file(None, &nostr_address_path)?;
            client.add_relays(&naddrs.extract_relays()).await;
            Some(naddrs)
        } else {
            None
        };

        client.add_relays(&self.to.relays).await;
        let relays_list = client.user_relays_list(user_pubk).await?;
        let author_read_relays =
            utils::add_read_relays(client.user_relays_list(user_pubk).await?.as_ref());
        client.add_relays(&author_read_relays).await;


        let reply_to = client
            .fetch_event(Filter::new().id(self.to.event_id))
            .await?
            .ok_or(N34Error::EventNotFound)?;
        let root = client.find_root(reply_to.clone()).await?;

        let repos_coordinate = if let Some(naddrs) = repo_naddrs {
            naddrs.into_coordinates()
        } else if let Some(ref root_event) = root {
            coordinates_from_root(root_event)?
        } else {
            return Err(N34Error::NotFoundRepo);
        };

        let repos = client.fetch_repos(&repos_coordinate).await?;
        let maintainers = repos.extract_maintainers();

        let quoted_content = if self.quote_to {
            Some(quote_reply_to_content(&client, &reply_to).await)
        } else {
            None
        };

        let content = utils::get_content(self.comment.as_ref(), quoted_content.as_ref(), ".txt")?;
        let content_details = client.parse_content(&content).await;

        let event = EventBuilder::comment(
            content,
            &reply_to,
            root.as_ref(),
            repos.first().and_then(|r| r.relays.first()).cloned(),
        )
        .dedup_tags()
        .pow(options.pow.unwrap_or_default())
        .tags(content_details.clone().into_tags())
        .build(user_pubk);

        let event_id = event.id.expect("There is an id");
        let write_relays = [
            relays,
            utils::add_write_relays(relays_list.as_ref()),
            // Merge repository announcement relays into write relays
            repos.extract_relays(),
            // Include read relays for each repository maintainer (if found)
            client.read_relays_from_users(&maintainers).await,
            // read relays of the root event and the reply to event
            {
                let (r1, r2) = future::join(
                    client.read_relays_from_user(reply_to.pubkey),
                    event_author_read_relays(&client, root.as_ref()),
                )
                .await;
                [r1, r2].concat()
            },
            content_details.write_relays.into_iter().collect(),
        ]
        .concat();

        tracing::trace!(relays = ?write_relays, "Write relays list");
        let (success, ..) = futures::join!(
            client.send_event_to(event, relays_list.as_ref(), &write_relays),
            client.broadcast(&reply_to, &author_read_relays),
            async {
                if let Some(root_event) = root {
                    let _ = client.broadcast(&root_event, &author_read_relays).await;
                }
            },
        );


        let nevent = utils::new_nevent(event_id, &success?)?;
        println!("Comment created: {nevent}");

        Ok(())
    }
}

/// Creates a quoted reply string in the format "On yyyy-mm-dd at hh:mm UTC,
/// {author} wrote:" followed by the event content. Uses display name if
/// available, otherwise falls back to a shortened npub string. Dates are
/// formatted in UTC.
async fn quote_reply_to_content(client: &NostrClient, quoted_event: &Event) -> String {
    let author_name = client.get_username(quoted_event.pubkey).await;

    let fdate = chrono::DateTime::from_timestamp(
        quoted_event
            .created_at
            .as_u64()
            .try_into()
            .unwrap_or(MAX_DATE),
        0,
    )
    .map(|datetime| datetime.format("On %F at %R UTC, ").to_string())
    .unwrap_or_default();

    format!(
        "{fdate}{author_name} wrote:\n> {}",
        quoted_event.content.trim().replace("\n", "\n> ")
    )
}

/// Gets the repository coordinate from a root Nostr event's tags.
/// The event must contain a coordinate tag with GitRepoAnnouncement kind.
fn coordinates_from_root(root: &Event) -> N34Result<Vec<Coordinate>> {
    let coordinates: Vec<Coordinate> = root
        .tags
        .coordinates()
        .filter(|c| c.kind == Kind::GitRepoAnnouncement)
        .cloned()
        .collect();

    if coordinates.is_empty() {
        return Err(N34Error::InvalidEvent(
            "The Git issue/patch does not specify a target repository".to_owned(),
        ));
    }

    Ok(coordinates)
}

/// Returns the event author read relays if found, otherwise an empty vector
async fn event_author_read_relays(client: &NostrClient, event: Option<&Event>) -> Vec<RelayUrl> {
    if let Some(root_event) = event {
        client.read_relays_from_user(root_event.pubkey).await
    } else {
        Vec::new()
    }
}
