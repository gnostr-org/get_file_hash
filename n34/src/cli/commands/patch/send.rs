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

use std::{fs, str::FromStr};

use clap::Args;
use futures::future;
use nostr::{
    event::{EventBuilder, EventId, Kind, Tag, TagKind, Tags, UnsignedEvent},
    hashes::sha1::Hash as Sha1Hash,
    key::PublicKey,
    nips::{nip01::Coordinate, nip10::Marker},
    types::RelayUrl,
};

use super::GitPatch;
use crate::{
    cli::{
        CliOptions,
        patch::{REVISION_ROOT_HASHTAG_CONTENT, ROOT_HASHTAG_CONTENT},
        traits::{CommandRunner, OptionNaddrOrSetVecExt, RelayOrSetVecExt},
        types::{NaddrOrSet, NostrEvent},
    },
    error::N34Result,
    nostr_utils::{
        NostrClient,
        traits::{NaddrsUtils, ReposUtils},
        utils,
    },
};

/// Prefix used for git patch alt.
const PATCH_ALT_PREFIX: &str = "git patch: ";

#[derive(Args, Debug)]
pub struct SendArgs {
    /// Repository address in `naddr` format (`naddr1...`), NIP-05 format
    /// (`4rs.nl/n34` or `_@4rs.nl/n34`), or a set name like `kernel`.
    ///
    /// If omitted, looks for a `nostr-address` file.
    #[arg(value_name = "NADDR-NIP05-OR-SET", long = "repo")]
    naddrs:         Option<Vec<NaddrOrSet>>,
    /// List of patch files to send (space separated).
    ///
    /// For p-tagging users, include them in the cover letter with
    /// `nostr:npub1...`.
    #[arg(value_name = "PATCH-PATH", required = true, value_parser = parse_patch_path)]
    patches:        Vec<GitPatch>,
    /// Original patch ID if this is a revision of it
    #[arg(long, value_name = "EVENT-ID")]
    original_patch: Option<NostrEvent>,
}

impl CommandRunner for SendArgs {
    async fn run(self, options: CliOptions) -> N34Result<()> {
        let naddrs = utils::check_empty_naddrs(utils::naddrs_or_file(
            self.naddrs.flat_naddrs(&options.config.sets)?,
            &utils::nostr_address_path()?,
        )?)?;

        let repo_coordinates = naddrs.clone().into_coordinates();
        let relays = options.relays.clone().flat_relays(&options.config.sets)?;
        let client = NostrClient::init(&options, &relays).await;
        let user_pubk = client.pubkey().await?;

        client.add_relays(&naddrs.extract_relays()).await;
        if let Some(original_patch) = &self.original_patch {
            client.add_relays(&original_patch.relays).await;
        }
        let relays_list = client.user_relays_list(user_pubk).await?;
        client
            .add_relays(&utils::add_read_relays(relays_list.as_ref()))
            .await;
        let repos = client.fetch_repos(&repo_coordinates).await?;
        let euc = repos.extract_euc();
        let maintainers = repos.extract_maintainers();
        client.add_relays(&repos.extract_relays()).await;

        let (events, events_write_relays) = make_patch_series(
            &client,
            self.patches,
            self.original_patch.as_ref().map(|e| e.event_id),
            repos.extract_relays().first().cloned(),
            repo_coordinates,
            euc,
            user_pubk,
        )
        .await?;

        let write_relays = [
            relays,
            repos.extract_relays(),
            events_write_relays,
            naddrs.extract_relays(),
            self.original_patch.map(|e| e.relays).unwrap_or_default(),
            utils::add_write_relays(relays_list.as_ref()),
            client.read_relays_from_users(&maintainers).await,
        ]
        .concat();

        tracing::trace!(write_relays = ?write_relays, "Write relays of the patches");
        let nevents = future::join_all(events.into_iter().map(|mut event| {
            async {
                let event_id = event.id();
                let subject = event
                    .tags
                    .find(TagKind::Alt)
                    .and_then(Tag::content)
                    .expect("There is an alt")
                    .replace(PATCH_ALT_PREFIX, "");
                client
                    .send_event_to(event, relays_list.as_ref(), &write_relays)
                    .await
                    .map(|r| Ok((subject, utils::new_nevent(event_id, &r)?)))?
            }
        }))
        .await
        .into_iter()
        .collect::<N34Result<Vec<_>>>()?;

        for (subject, nevent) in nevents {
            println!("Created '{subject}': {nevent}");
        }

        Ok(())
    }
}

fn parse_patch_path(patch_path: &str) -> Result<GitPatch, String> {
    tracing::debug!("Parsing patch file `{patch_path}`");
    let patch_content = fs::read_to_string(patch_path)
        .map_err(|err| format!("Failed to read patch file `{patch_path}`: {err}"))?;
    GitPatch::from_str(&patch_content)
}

async fn make_patch_series(
    client: &NostrClient,
    patches: Vec<GitPatch>,
    original_patch: Option<EventId>,
    relay_hint: Option<RelayUrl>,
    repo_coordinates: Vec<Coordinate>,
    euc: Option<&Sha1Hash>,
    author_pkey: PublicKey,
) -> N34Result<(Vec<UnsignedEvent>, Vec<RelayUrl>)> {
    let mut write_relays = Vec::new();
    let mut patch_series = Vec::new();
    let mut patches = patches.into_iter();
    let root_patch = patches.next().expect("Patches can't be empty");
    let (root_event, root_relays) = make_patch(
        client,
        root_patch,
        None,
        original_patch,
        relay_hint.as_ref(),
        &repo_coordinates,
        euc,
        author_pkey,
    )
    .await;
    write_relays.extend(root_relays);
    let root_id = *root_event.id.as_ref().expect("There is an id");
    let mut previous_patch = root_id;
    patch_series.push(root_event);

    for patch in patches {
        let (patch_event, patch_relays) = make_patch(
            client,
            patch,
            Some(root_id),
            Some(previous_patch),
            relay_hint.as_ref(),
            &repo_coordinates,
            euc,
            author_pkey,
        )
        .await;
        previous_patch = patch_event.id.expect("there is an id");
        write_relays.extend(patch_relays);
        patch_series.push(patch_event);
    }

    Ok((patch_series, write_relays))
}

#[allow(clippy::too_many_arguments)]
async fn make_patch(
    client: &NostrClient,
    patch: GitPatch,
    root: Option<EventId>,
    reply_to: Option<EventId>,
    write_relay: Option<&RelayUrl>,
    repo_coordinates: &[Coordinate],
    euc: Option<&Sha1Hash>,
    author_pkey: PublicKey,
) -> (UnsignedEvent, Vec<RelayUrl>) {
    let content_details = client.parse_content(&patch.body).await;
    let content_relays = content_details.write_relays.clone();
    // NIP-34 compliance requires referencing the previous patch using `NIP-10 e
    // reply`. However, this fails for the second patch when
    // `EventBuilder::dedup_tags` is enabled because:
    // 1. The tag is treated as a duplicate based on its content (the root ID).
    // 2. The second patch would reply to the root twice:
    //    - First with the 'root' marker
    //    - Then with the 'reply' marker
    // The `EventBuilder::dedup_tags` function then removes the 'reply' marker as a
    // duplicate.
    let mut safe_dedup_tags = Tags::new();
    safe_dedup_tags.push(Tag::alt(format!("{PATCH_ALT_PREFIX}{}", patch.subject)));
    safe_dedup_tags.push(Tag::description(patch.subject));
    safe_dedup_tags.extend(content_details.into_tags());
    safe_dedup_tags.extend(
        repo_coordinates
            .iter()
            .map(|c| Tag::coordinate(c.clone(), None)),
    );
    safe_dedup_tags.extend(
        repo_coordinates
            .iter()
            .map(|c| Tag::public_key(c.public_key)),
    );
    if let Some(euc) = euc {
        safe_dedup_tags.push(Tag::reference(euc.to_string()));
    }
    safe_dedup_tags.dedup();
    let mut event_builder = EventBuilder::new(Kind::GitPatch, patch.inner).tags(safe_dedup_tags);

    // If the root is None, this indicates we're handling the root event
    if let Some(root_id) = root {
        event_builder =
            event_builder.tag(utils::event_reply_tag(&root_id, write_relay, Marker::Root));
    } else {
        event_builder = event_builder.tag(Tag::hashtag(ROOT_HASHTAG_CONTENT));
    }

    // Handles the case where there is a patch to reply to but no root. This
    // indicates we are processing a revision, as the root revision should reply
    // directly to the original patch.
    if let Some(reply_to_id) = reply_to {
        if root.is_none() {
            event_builder = event_builder.tags([
                utils::event_reply_tag(&reply_to_id, write_relay, Marker::Reply),
                Tag::hashtag(REVISION_ROOT_HASHTAG_CONTENT),
            ]);
        } else {
            event_builder = event_builder.tag(utils::event_reply_tag(
                &reply_to_id,
                write_relay,
                Marker::Reply,
            ));
        }
    }
    (
        event_builder.build(author_pkey),
        content_relays.into_iter().collect(),
    )
}
