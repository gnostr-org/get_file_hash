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

/// Extension traits for nostr types.
pub mod traits;
/// Utility functions for nostr.
pub mod utils;

use std::{collections::HashSet, time::Duration};

use futures::future;
use nostr::{
    event::{Event, EventId, Kind, Tag, TagKind, TagStandard, Tags, UnsignedEvent},
    filter::Filter,
    key::PublicKey,
    nips::{
        nip01::{Coordinate, Metadata},
        nip19::ToBech32,
        nip22,
        nip34::GitRepositoryAnnouncement,
    },
    parser::NostrParser,
    types::RelayUrl,
};
use nostr_sdk::{Client, ClientOptions};
use traits::TokenUtils;

use crate::{
    cli::{CliOptions, issue::IssueStatus, patch::PatchStatus},
    error::{N34Error, N34Result},
};

/// Timeout duration for the client.
const CLIENT_TIMEOUT: Duration = Duration::from_millis(1500);
/// Length of a Nostr npub (public key) in characters.
const NPUB_LEN: usize = 63;

/// Parsed content details
#[derive(Clone)]
pub struct ContentDetails {
    /// Public keys of users mentioned in the content.
    pub p_tagged:     HashSet<PublicKey>,
    /// Event IDs and optional relay URLs for quoted events.
    pub quotes:       HashSet<(EventId, Option<RelayUrl>)>,
    /// Hashtags found in the content.
    pub hashtags:     HashSet<String>,
    /// Relays where mentioned users and quoted authors are read.
    pub write_relays: HashSet<RelayUrl>,
}

/// A client for interacting with the Nostr relays
#[derive(Clone)]
pub struct NostrClient {
    /// The underlying Nostr client implementation
    pub client: Client,
}

impl ContentDetails {
    /// Create a new [`ContentDetails`] instance
    pub fn new(
        users: impl IntoIterator<Item = PublicKey>,
        quotes: impl IntoIterator<Item = (EventId, Option<RelayUrl>)>,
        hashtags: impl IntoIterator<Item = String>,
        write_relays: impl IntoIterator<Item = RelayUrl>,
    ) -> Self {
        Self {
            p_tagged:     HashSet::from_iter(users),
            quotes:       HashSet::from_iter(quotes),
            hashtags:     HashSet::from_iter(hashtags),
            write_relays: HashSet::from_iter(write_relays),
        }
    }

    /// Converts the instance into a list of tags including hashtags, p-tagged
    /// users, and quoted events.
    pub fn into_tags(self) -> Tags {
        let mut tags = Tags::new();
        tags.extend(self.hashtags.into_iter().map(Tag::hashtag));
        tags.extend(self.p_tagged.into_iter().map(Tag::public_key));
        tags.extend(self.quotes.into_iter().map(|(event_id, relay_url)| {
            // TODO: Add the author public key if we know it
            Tag::from_standardized(TagStandard::Quote {
                event_id,
                relay_url,
                public_key: None,
            })
        }));
        tags
    }
}

impl NostrClient {
    /// Creates a new [`NostrClient`] with the given client and options.
    const fn new(client: Client) -> Self {
        Self { client }
    }

    /// Initializes a new [`NostrClient`] instance and connects to the specified
    /// relays.
    pub async fn init(options: &CliOptions, relays: &[RelayUrl]) -> Self {
        let mut client_builder =
            Client::builder().opts(ClientOptions::new().verify_subscriptions(true));

        if let Ok(Some(signer)) = options.signer().await {
            client_builder = client_builder.signer(signer);
        }

        let client = Self::new(client_builder.build());

        client.add_relays(relays).await;
        client
    }

    //// Returns the users public key
    pub async fn pubkey(&self) -> N34Result<PublicKey> {
        self.client
            .signer()
            .await?
            .get_public_key()
            .await
            .map_err(N34Error::SignerError)
    }

    /// Add relays and connect to them
    pub async fn add_relays(&self, relays: &[RelayUrl]) {
        if relays.is_empty() {
            return;
        }

        let mut tasks = Vec::new();
        for relay in relays {
            let relay = relay.clone();
            let client = self.client.clone();
            tasks.push(tokio::spawn(async move {
                client
                    .add_relay(&relay)
                    .await
                    .expect("It's a valid relay url");
                if let Err(err) = client.try_connect_relay(&relay, CLIENT_TIMEOUT).await {
                    tracing::error!("Failed to connect to relay '{relay}': {err}");
                }
            }));
        }
        future::join_all(tasks).await;
    }

    /// Add a relay hint and connect to it
    pub async fn add_relay_hint(&self, hint: Option<RelayUrl>) {
        if let Some(relay) = hint {
            self.add_relays(&[relay]).await
        }
    }

    /// broadcast an event to the given relays
    pub async fn broadcast(&self, event: &Event, relays: &[RelayUrl]) -> N34Result<()> {
        self.client.send_event_to(relays, event).await?;
        Ok(())
    }

    /// Broadcasts an unsigned event to given relays, optionally broadcast the
    /// relays list event. Returns URLs of relays that successfully received
    /// the event.
    pub async fn send_event_to(
        &self,
        mut event: UnsignedEvent,
        relays_list: Option<&Event>,
        relays: &[RelayUrl],
    ) -> N34Result<Vec<RelayUrl>> {
        self.add_relays(relays).await;
        let event_id = event.id();

        let (result, ..) = futures::join!(
            async {
                N34Result::Ok(
                    self.client
                        .send_event_to(relays, &event.sign(&self.client.signer().await?).await?)
                        .await?,
                )
            },
            async {
                if let Some(event) = relays_list {
                    let _ = self.client.send_event_to(relays, event).await;
                }
            }
        );
        let result = result?;

        for relay in &result.success {
            tracing::info!(event_id = %event_id, relay = %relay, "Event sent successfully");
        }
        for (relay, reason) in &result.failed {
            tracing::warn!(event_id = %event_id, relay = %relay, reason = %reason, "Failed to send event");
        }

        Ok(result.success.into_iter().collect())
    }

    /// Fetches the first event matching the given filter, or None if no event
    /// is found.
    pub async fn fetch_event(&self, filter: Filter) -> N34Result<Option<Event>> {
        Ok(self
            .client
            .fetch_events(filter.limit(1), CLIENT_TIMEOUT)
            .await?
            .first_owned())
    }

    /// Fetches the events matching the given filter
    pub async fn fetch_events(&self, filter: Filter) -> N34Result<Vec<Event>> {
        // Multiply timeout by 5 to account for multiple events being fetched
        Ok(self
            .client
            .fetch_events(filter, CLIENT_TIMEOUT * 5)
            .await?
            .to_vec())
    }

    /// Try to fetch the repositories and returns them
    pub async fn fetch_repos(
        &self,
        repo_naddrs: &[Coordinate],
    ) -> N34Result<Vec<GitRepositoryAnnouncement>> {
        future::join_all(repo_naddrs.iter().map(|c| {
            async {
                self.fetch_event(
                    Filter::new()
                        .author(c.public_key)
                        .identifier(&c.identifier)
                        .kind(Kind::GitRepoAnnouncement),
                )
                .await?
                .map(|e| utils::event_into_repo(e, &c.identifier))
                .ok_or(N34Error::NotFoundRepo)
            }
        }))
        .await
        .into_iter()
        .collect()
    }

    /// Fetch the patch by the given id. None if not found
    pub async fn fetch_patch(&self, patch_id: EventId) -> N34Result<Event> {
        self.fetch_event(Filter::new().id(patch_id).kind(Kind::GitPatch))
            .await?
            .ok_or(N34Error::CanNotFoundPatch)
    }

    /// Returns the username for a given public key. If no username is found,
    /// falls back to a shortened version of the public key.
    pub async fn get_username(&self, user: PublicKey) -> String {
        self.fetch_event(Filter::new().kind(Kind::Metadata).author(user))
            .await
            .ok()
            .flatten()
            .and_then(|e| Metadata::try_from(&e).ok())
            .and_then(|m| m.display_name.or(m.name))
            .unwrap_or_else(|| {
                let pubkey = user.to_bech32().expect("The error is `Infallible`");
                format!("{}...{}", &pubkey[..8], &pubkey[NPUB_LEN - 8..])
            })
    }

    /// Get the latest status of an issue by its ID, only considering status
    /// events from authorized_pubkeys. If no valid status event is found,
    /// defaults to Open.
    pub async fn fetch_issue_status(
        &self,
        issue_id: EventId,
        authorized_pubkeys: Vec<PublicKey>,
    ) -> N34Result<IssueStatus> {
        self.fetch_events(
            Filter::new()
                .event(issue_id)
                .kinds([
                    Kind::GitStatusOpen,
                    Kind::GitStatusApplied,
                    Kind::GitStatusClosed,
                ])
                .authors(utils::dedup(authorized_pubkeys.into_iter())),
        )
        .await?
        .into_iter()
        .max_by_key(|e| e.created_at)
        .map(|status| IssueStatus::try_from(status.kind))
        .unwrap_or_else(|| Ok(IssueStatus::Open))
    }

    /// Gets the status of a patch. If it's a revision patch, checks if it's
    /// closed when the root patch is already merged/applied but doesn't
    /// reference this revision. Defaults to Open status if no status event
    /// is found.
    pub async fn fetch_patch_status(
        &self,
        root_patch: EventId,
        root_revision: Option<EventId>,
        authorized_pubkeys: Vec<PublicKey>,
    ) -> N34Result<PatchStatus> {
        let (root_status, event_tags) = self
            .fetch_events(
                Filter::new()
                    .event(root_patch)
                    .kinds([
                        Kind::GitStatusOpen,
                        Kind::GitStatusApplied,
                        Kind::GitStatusClosed,
                        Kind::GitStatusDraft,
                    ])
                    .authors(utils::dedup(authorized_pubkeys.into_iter())),
            )
            .await?
            .into_iter()
            .max_by_key(|e| e.created_at)
            .map(|status| N34Result::Ok((PatchStatus::try_from(status.kind)?, status.tags)))
            .unwrap_or_else(|| Ok((PatchStatus::Open, Tags::new())))?;

        if let Some(revision_id) = root_revision
            && root_status.is_merged_or_applied()
            && !event_tags
                .filter(TagKind::e())
                .any(|t| t.is_reply() && t.content().is_some_and(|c| c == revision_id.to_hex()))
        {
            return Ok(PatchStatus::Closed);
        }

        Ok(root_status)
    }

    pub async fn fetch_patch_series(
        &self,
        root_patch_id: EventId,
        root_patch_author: PublicKey,
    ) -> N34Result<Vec<Event>> {
        Ok(self
            .fetch_events(
                Filter::new()
                    .kind(Kind::GitPatch)
                    .author(root_patch_author)
                    .event(root_patch_id),
            )
            .await?
            .into_iter()
            .filter(|e| {
                e.tags.iter().any(|t| {
                    t.is_root() && t.content().is_some_and(|c| c == root_patch_id.to_hex())
                })
            })
            .collect())
    }

    /// Finds the root issue or patch for a given event. If the event is already
    /// a root (issue/patch), returns it directly. For comments, follows
    /// parent/root references until finding the root or failing. Returns
    /// None if no root can be found.
    pub async fn find_root(&self, mut event: Event) -> N34Result<Option<Event>> {
        if !matches!(event.kind, Kind::GitIssue | Kind::GitPatch | Kind::Comment) {
            return Err(N34Error::CanNotReplyToEvent);
        }

        loop {
            if matches!(event.kind, Kind::GitIssue | Kind::GitPatch) {
                return Ok(Some(event));
            }

            if let Some(nip22::CommentTarget::Event { id, relay_hint, .. }) =
                nip22::extract_root(&event)
            {
                self.add_relay_hint(relay_hint.cloned()).await;
                let root_event = self.fetch_event(Filter::new().id(*id)).await?;
                if let Some(ref root_event) = root_event
                    && !matches!(root_event.kind, Kind::GitIssue | Kind::GitPatch)
                {
                    return Err(N34Error::CanNotReplyToEvent);
                }
                return Ok(root_event);
            } else if let Some(nip22::CommentTarget::Event { id, relay_hint, .. }) =
                nip22::extract_parent(&event)
            {
                self.add_relay_hint(relay_hint.cloned()).await;
                if let Ok(Some(parent_event)) = self.fetch_event(Filter::new().id(*id)).await {
                    event = parent_event;
                    continue;
                }
            }

            // Break if: no root/parent tags found, parent/root event fetch failed
            break;
        }

        Ok(None)
    }

    /// Fetches the relay list (kind 10002) for the given user. Returns None if
    /// no relays are found.
    pub async fn user_relays_list(&self, user: PublicKey) -> N34Result<Option<Event>> {
        self.fetch_event(Filter::new().author(user).kind(Kind::RelayList))
            .await
    }

    /// Gets the author of the specified event, if found.
    pub async fn event_author(&self, event_id: EventId) -> N34Result<Option<PublicKey>> {
        Ok(self
            .fetch_event(Filter::new().id(event_id))
            .await?
            .map(|e| e.pubkey))
    }

    /// Returns the read relays of the given user if found, otherwise empty
    /// vector
    pub async fn read_relays_from_user(&self, user: PublicKey) -> Vec<RelayUrl> {
        utils::add_read_relays(self.user_relays_list(user).await.ok().flatten().as_ref())
    }

    /// Returns the read relays of the given users if found, otherwise empty
    /// vector
    pub async fn read_relays_from_users(&self, users: &[PublicKey]) -> Vec<RelayUrl> {
        self.fetch_events(
            Filter::new()
                .kind(nostr::event::Kind::RelayList)
                .authors(utils::dedup(users.iter().copied())),
        )
        .await
        .unwrap_or_default()
        .into_iter()
        .flat_map(|e| utils::add_read_relays(Some(&e)))
        .collect()
    }

    /// Parse the given content and returns the details that inside it
    pub async fn parse_content(&self, content: &str) -> ContentDetails {
        let mut write_relays = Vec::new();
        let tokens = NostrParser::new().parse(content).collect::<Vec<_>>();

        let mut p_tagged_users = tokens
            .iter()
            .filter_map(TokenUtils::extract_public_key)
            .collect::<Vec<_>>();
        let quotes = tokens
            .iter()
            .filter_map(TokenUtils::extract_event_id)
            .collect::<Vec<_>>();
        let hashtags = tokens
            .iter()
            .filter_map(TokenUtils::extract_hashtag)
            .collect::<Vec<_>>();

        for (user, relays) in &p_tagged_users {
            self.add_relays(relays).await;
            write_relays.extend(self.read_relays_from_user(*user).await);
        }
        for (event_id, relays) in &quotes {
            self.add_relays(relays).await;
            // Add the event author to the p-tagged users
            if let Ok(Some(author)) = self.event_author(*event_id).await {
                p_tagged_users.push((author, Vec::new()));
                write_relays.extend(self.read_relays_from_user(author).await);
            }
        }

        ContentDetails::new(
            p_tagged_users.into_iter().map(|(p, _)| p),
            quotes.into_iter().map(|(e, r)| (e, r.first().cloned())),
            hashtags,
            write_relays,
        )
    }
}
