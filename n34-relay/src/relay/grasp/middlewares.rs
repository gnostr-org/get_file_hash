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

use either::Either;
use nostr::{
    event::{Event, Kind, TagKind},
    filter::Filter,
    nips::nip19::ToBech32,
    util::BoxedFuture,
};

use crate::relay::{MiddlewareState, grasp::git_command::GraspGitCommand};

/// Creates a repo if received a repository announcements contains the relay
/// domain in `clone` tag and `relay` tag
pub fn repo_creator<'a>(
    repo_or_query: Either<&'a Filter, &'a Event>,
    state: Arc<MiddlewareState>,
) -> BoxedFuture<'a, ()> {
    Box::pin(async move {
        let Either::Right(event) = repo_or_query else {
            return;
        };

        if event.kind != Kind::GitRepoAnnouncement {
            return;
        }

        let Some(repo_name) = event.tags.identifier() else {
            return;
        };

        let repo_description = event
            .tags
            .find(TagKind::Description)
            .and_then(|t| t.content())
            .unwrap_or("N/A");

        if let Err(err) = GraspGitCommand::new(
            &state.config.grasp.git_path,
            &state
                .config
                .grasp
                .repos_path
                .join(event.pubkey.to_bech32().expect("Infallible"))
                .join(repo_name)
                .with_extension("git"),
        )
        .new_bare(repo_description)
        .await
        {
            tracing::error!("{err}");
        }
    })
}
