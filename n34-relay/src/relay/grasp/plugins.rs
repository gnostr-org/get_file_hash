// n34-relay - A nostr GRASP relay implementation
// Copyright (C) 2025 Awiteb <a@4rs.nl>
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published
// by the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program. If not, see <https://gnu.org/licenses/agpl-3.0>.

use std::sync::Arc;

use nostr::{
    event::{Event, EventId, Kind, Tag, TagKind, TagStandard},
    filter::{Alphabet, Filter, SingleLetterTag},
    message::MachineReadablePrefix,
    nips::{nip01::Coordinate, nip19::ToBech32},
    util::BoxedFuture,
};
use nostr_database::NostrDatabase;
use nostr_relay_builder::builder::WritePolicyResult;

use crate::{
    ext_traits::WritePolicyResultExt,
    git_server::utils as git_utils,
    relay::plugins_manager::RelayPlugin,
    utils,
};

/// Determines if a repository announcement event is valid.
pub struct ValidateRepoEvent;

/// Checks if a repository is a GRASP repository and rejects it if not.
pub struct GraspRepo {
    domain: String,
}

/// Accept events that tag or tagged by accepted git repository announcement or
/// accepted issues or patches
pub struct AcceptMention {
    pub db: Arc<dyn NostrDatabase>,
}

/// Rejects the repository state announcement if we don't have the repository
pub struct RejectRepoState {
    pub db: Arc<dyn NostrDatabase>,
}

/// Checks if the repository state announcement is valid.
/// This ensures the `d` tag is correct, `HEAD` is present, and the `HEAD` ref
/// exists.
pub struct ValidateRepoState;

impl GraspRepo {
    /// Constructs a new [GraspRepo] plugin
    #[inline]
    pub fn new(relay_domain: impl Into<String>) -> Self {
        GraspRepo {
            domain: relay_domain.into(),
        }
    }
}

impl AcceptMention {
    #[inline]
    /// Constructs a new [AcceptMention] plugin
    pub fn new(db: Arc<dyn NostrDatabase>) -> Self {
        Self { db }
    }

    /// Checks if any event matching the given filter in the database.
    async fn db_contains(&self, filter: Filter) -> bool {
        match self.db.count(filter.clone()).await {
            Ok(count) => count != 0,
            Err(err) => {
                tracing::error!(error = %err, filter = ?filter, "Failed to query the database");
                false
            }
        }
    }
}

impl RejectRepoState {
    #[inline]
    /// Constructs a new [RejectRepoState] plugin
    pub fn new(db: Arc<dyn NostrDatabase>) -> Self {
        Self { db }
    }
}

impl RelayPlugin for ValidateRepoEvent {
    fn check_event<'a>(&'a self, event: &'a Event) -> BoxedFuture<'a, Option<WritePolicyResult>> {
        Box::pin(async {
            if event.kind != Kind::GitRepoAnnouncement {
                return None;
            }

            // At least one relay
            if event
                .tags
                .find(TagKind::Relays)
                .is_none_or(|relays| relays.content().is_none())
            {
                return Some(WritePolicyResult::blocked_reject(
                    "No relays in the repository announcement",
                ));
            }

            // At least one clone url
            if event
                .tags
                .find(TagKind::Clone)
                .is_none_or(|clones| clones.content().is_none())
            {
                return Some(WritePolicyResult::blocked_reject(
                    "No clone urls in the repository announcement",
                ));
            }

            if event.tags.filter(TagKind::d()).count() > 1 {
                return Some(WritePolicyResult::blocked_reject(
                    "More than one `d` tag in the repository announcement",
                ));
            }

            let Some(repo_name) = event.tags.identifier() else {
                return Some(WritePolicyResult::blocked_reject(
                    "The repository announcement must contains `d` tag",
                ));
            };

            if repo_name.chars().count() > 30 {
                return Some(WritePolicyResult::blocked_reject(
                    "Repository name exceeds maximum length of 30 characters",
                ));
            }

            // Check if it's a valid name. All ascii and no whitespace
            if repo_name
                .chars()
                .any(|c| !c.is_ascii_alphanumeric() && c != '-' && c != '_')
            {
                return Some(WritePolicyResult::blocked_reject(format!(
                    "Invalid repository name '{repo_name}'. Repository names can only contain \
                     ASCII letters, numbers, hyphens (-), and underscores (_)."
                )));
            }

            Some(WritePolicyResult::Accept)
        })
    }
}

impl RelayPlugin for GraspRepo {
    fn check_event<'a>(&'a self, event: &'a Event) -> BoxedFuture<'a, Option<WritePolicyResult>> {
        Box::pin(async {
            if event.kind != Kind::GitRepoAnnouncement {
                return None;
            }

            let repo_name = event
                .tags
                .identifier()
                .expect("Verified in ValidateRepoEvent");

            let relays = event
                .tags
                .find(TagKind::Relays)
                .map(|t| &t.as_slice()[1..])
                .expect("Verified in ValidateRepoEvent");

            let clones = event
                .tags
                .find(TagKind::Clone)
                .map(|t| &t.as_slice()[1..])
                .expect("Verified in ValidateRepoEvent");

            let repo_clone_url = format!(
                "{}/{}/{repo_name}.git",
                self.domain,
                event.pubkey.to_bech32().expect("Infallible")
            );

            if !relays
                .iter()
                .any(|relay| utils::remove_proto(relay).starts_with(&self.domain))
            {
                return Some(WritePolicyResult::blocked_reject(format!(
                    "`{}` relay is not listed in the 'relays' tag of the announcement",
                    self.domain
                )));
            }

            if !clones
                .iter()
                .any(|clone_url| utils::remove_proto(clone_url) == repo_clone_url)
            {
                return Some(WritePolicyResult::blocked_reject(format!(
                    "`{}` relay does not match any URLs in the 'clone' tag of the announcement",
                    self.domain
                )));
            }

            Some(WritePolicyResult::Accept)
        })
    }
}

impl RelayPlugin for AcceptMention {
    fn check_event<'a>(&'a self, event: &'a Event) -> BoxedFuture<'a, Option<WritePolicyResult>> {
        Box::pin(async {
            // from GRASP protocol:
            // "MUST accept other events that tag, or are tagged by, either:
            // 1. accepted git repository announcements; or
            // 2. accepted issues or patches"

            // Check if the event tag a repository announcement
            if self
                .db_contains(Filter::new().coordinates(repos_coordinate(event)))
                .await
            {
                return Some(WritePolicyResult::Accept);
            }

            // Check if the event tag a patch or an issue
            if self
                .db_contains(
                    Filter::new()
                        .kinds([Kind::GitPatch, Kind::GitIssue])
                        .ids(tagged_events(event)),
                )
                .await
            {
                return Some(WritePolicyResult::Accept);
            }

            // Check if the event is tagged by a patch or an issue. By either `e` tag or `q`
            // tag
            if self
                .db_contains(
                    Filter::new()
                        .kinds([Kind::GitPatch, Kind::GitIssue])
                        .event(event.id),
                )
                .await
                || self
                    .db_contains(
                        Filter::new()
                            .kinds([Kind::GitPatch, Kind::GitIssue])
                            .custom_tag(SingleLetterTag::lowercase(Alphabet::Q), event.id),
                    )
                    .await
            {
                return Some(WritePolicyResult::Accept);
            }

            None
        })
    }
}

impl RelayPlugin for ValidateRepoState {
    fn check_event<'a>(&'a self, event: &'a Event) -> BoxedFuture<'a, Option<WritePolicyResult>> {
        Box::pin(async {
            if event.kind != Kind::RepoState {
                return None;
            }

            if event.tags.filter(TagKind::d()).count() > 1 {
                return Some(WritePolicyResult::blocked_reject(
                    "Invalid repository state announcement. More than one `d` tag",
                ));
            }

            let Some(repo_name) = event.tags.identifier() else {
                return Some(WritePolicyResult::blocked_reject(
                    "Invalid repository state announcement. No `d` tag",
                ));
            };

            if repo_name.chars().count() > 30 {
                return Some(WritePolicyResult::blocked_reject(
                    "Repository name exceeds maximum length of 30 characters",
                ));
            }

            let Some(mut head) = event.tags.find(TagKind::Head).and_then(Tag::content) else {
                return Some(WritePolicyResult::blocked_reject(
                    "No `HEAD` tag in the repository state announcement",
                ));
            };

            if !head.starts_with("ref: refs/heads/") {
                return Some(WritePolicyResult::blocked_reject(
                    "The `HEAD` tag must start with `ref: refs/heads/`",
                ));
            }

            for (ref_name, commit) in git_utils::extract_refs(event) {
                if !utils::is_valid_sha1(commit) {
                    return Some(WritePolicyResult::blocked_reject(format!(
                        "`{ref_name}` has an invalid sha1 commit id"
                    )));
                }
            }

            head = head.trim_start_matches("ref: ").trim();
            if !git_utils::extract_refs(event).any(|(ref_name, _)| ref_name == head) {
                return Some(WritePolicyResult::blocked_reject(format!(
                    "No ref for the head `{head}`"
                )));
            }

            Some(WritePolicyResult::Accept)
        })
    }
}

impl RelayPlugin for RejectRepoState {
    fn check_event<'a>(&'a self, event: &'a Event) -> BoxedFuture<'a, Option<WritePolicyResult>> {
        Box::pin(async {
            if event.kind != Kind::RepoState {
                return None;
            }

            let repo_name = event
                .tags
                .identifier()
                .expect("Verified in ValidateRepoState");

            // Get the repositories with the same identifier
            let events = match self
                .db
                .query(
                    Filter::new()
                        .kind(Kind::GitRepoAnnouncement)
                        .identifier(repo_name)
                        .limit(100),
                )
                .await
            {
                Ok(events) => events,
                Err(err) => {
                    tracing::error!("Database error: {err}");
                    return Some(WritePolicyResult::reject(
                        MachineReadablePrefix::Error,
                        "Database error",
                    ));
                }
            };

            // Accept the state announcement if the repository state author is an author or
            // a maintainer in any of the repositories
            if events.iter().any(|repo_announcement| {
                repo_announcement.pubkey == event.pubkey
                    || utils::get_maintainers(repo_announcement)
                        .any(|maintainer| maintainer == &event.pubkey)
            }) {
                Some(WritePolicyResult::Accept)
            } else {
                Some(WritePolicyResult::blocked_reject(
                    "You don't have a repository for this state announcement in the relay.",
                ))
            }
        })
    }
}

/// Extracts up to 20 repository coordinates from an event's tags.
/// Only includes coordinates marked as Git repository announcements.
fn repos_coordinate(event: &Event) -> impl Iterator<Item = &Coordinate> {
    event
        .tags
        .as_slice()
        .iter()
        .filter_map(|t| {
            if let TagStandard::Coordinate { coordinate, .. } = t.as_standardized()?
                && coordinate.kind == Kind::GitRepoAnnouncement
            {
                return Some(coordinate);
            }
            None
        })
        .take(20)
}

/// Extract up to 20 tagged events from an event's tags.
fn tagged_events(event: &Event) -> impl Iterator<Item = EventId> {
    event
        .tags
        .as_slice()
        .iter()
        .filter_map(|t| {
            if let TagStandard::Event { event_id, .. } = t.as_standardized()? {
                return Some(*event_id);
            }
            None
        })
        .take(20)
}
