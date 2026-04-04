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

use std::{collections::HashMap, sync::Arc};

use axum::{body::Body, response::Response};
use hyper::{HeaderMap, StatusCode};
use nostr::{
    event::{Event, Kind},
    filter::Filter,
};
use nostr_database::NostrDatabase;

use crate::git_server::{PublicKeyAndRepoPath, ref_update_pkt_line::RefUpdatePkt};

/// unpack-status = PKT-LINE("unpack" SP unpack-result)
/// unpack-result = "ok" / error-msg
const UNPACK_OK: &[u8] = b"unpack ok";

/// Checks if the provided headers contain the `Git-Protocol: version=2` header.
pub fn contains_git_v2(headers: &HeaderMap) -> bool {
    headers
        .get("Git-Protocol")
        .is_some_and(|value| value == "version=2")
}

/// Add the data length before it. `{length}{data}`
fn pkt_line(data: &[u8]) -> Vec<u8> {
    let mut result = format!("{:04x}", data.len() + 4).into_bytes();
    result.extend_from_slice(data);
    result
}

/// Make a pkt_line with a side-band. `{length}{channel}pkt_line({data})`
fn sideband_pkt_line(channel: u8, data: &[u8]) -> Vec<u8> {
    let pktline = if data == b"0000" {
        data
    } else {
        &pkt_line(data)
    };

    // 4 bytes for length + 1 byte for channel + pktline
    let mut result = format!("{:04x}", pktline.len() + 5).into_bytes();
    result.push(channel);
    result.extend_from_slice(pktline);
    result
}

/// Formats an error message using Git's pkt-line report-status.
fn git_errors_response(
    refs_errors: &HashMap<&str, &'static str>,
    all_refs: &[RefUpdatePkt],
    capabilities: &str,
) -> Vec<u8> {
    let mut response = Vec::new();
    let caps_contains = |cap| capabilities.split(" ").any(|c| c == cap);
    let use_sideband = caps_contains("side-band-64k") || caps_contains("side-band");

    // report-status = unpack-status
    //                 1*(command-status)
    //                 flush-pkt
    if caps_contains("report-status") || caps_contains("report-status-v2") {
        // command-status = command-ok / command-fail
        // command-ok     = PKT-LINE("ok" SP refname)
        // command-fail   = PKT-LINE("ng" SP refname SP error-msg)
        let ng_line = |ref_name, msg| format!("ng {ref_name} {msg}");
        let ok_line = |ref_name| format!("ok {ref_name}");

        if use_sideband {
            response.extend_from_slice(&sideband_pkt_line(1, UNPACK_OK));
            for ref_update in all_refs {
                if let Some(err) = refs_errors.get(ref_update.ref_name) {
                    response.extend_from_slice(&sideband_pkt_line(
                        1,
                        ng_line(&ref_update.ref_name, err).as_bytes(),
                    ));
                } else {
                    response.extend_from_slice(&sideband_pkt_line(
                        1,
                        ok_line(ref_update.ref_name).as_bytes(),
                    ));
                }
            }
            // side-band flush-pkt
            response.extend_from_slice(&sideband_pkt_line(1, b"0000"));
            // flush-pkt
            response.extend_from_slice(b"0000");
        } else {
            response.extend_from_slice(&pkt_line(UNPACK_OK));
            for ref_update in all_refs {
                if let Some(err) = refs_errors.get(ref_update.ref_name) {
                    response.extend_from_slice(&pkt_line(
                        ng_line(&ref_update.ref_name, err).as_bytes(),
                    ));
                } else {
                    response.extend_from_slice(&pkt_line(ok_line(ref_update.ref_name).as_bytes()));
                }
            }
            // flush-pkt
            response.extend_from_slice(b"0000");
        }
    }
    response
}

/// Creates an error response that Git clients will display to users.
pub fn git_receive_pack_error(
    refs_errors: &HashMap<&str, &'static str>,
    all_refs: &[RefUpdatePkt],
    capabilities: &str,
) -> Response {
    let body = git_errors_response(refs_errors, all_refs, capabilities);

    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/x-git-receive-pack-result")
        .body(Body::from(body))
        .expect("valid response")
}

/// Extract the refs from the repository state event
#[inline]
pub fn extract_refs(repo_state: &Event) -> impl Iterator<Item = (&str, &str)> {
    repo_state.tags.iter().filter_map(|tag| {
        let tag_kind = tag.as_slice().first()?.as_str();
        if (tag_kind.starts_with("refs/heads/") || tag_kind.starts_with("refs/tags/"))
            && let Some(commit_id) = tag.content()
        {
            return Some((tag_kind, commit_id));
        }
        None
    })
}

/// Check if the ref match with the repository state
pub fn check_ref_update(ref_update: &RefUpdatePkt, repo_state: &Event) -> Result<(), &'static str> {
    let ref_commit = extract_refs(repo_state)
        .find_map(|(name, commit)| (ref_update.ref_name == name).then_some(commit));

    // Return an error if the ref is not found and the operation is not a delete
    if ref_commit.is_none() && !ref_update.is_delete() {
        return Err(
            "Reference not found. The reference must exist in the repository state to update it",
        );
    }

    // Return an error if attempting to delete a ref that is still present in the
    // repository state
    if ref_commit.is_some() && ref_update.is_delete() {
        return Err("Cannot delete reference: it still exists in the repository state");
    }

    // Return an error if the ref's current commit doesn't match the new commit
    if let Some(commit) = ref_commit
        && commit != ref_update.new_commit
    {
        return Err(
            "Commit mismatch: the new commit doesn't match the repository state for this reference",
        );
    }

    // Accept if:
    // - ref is not found the the commit is delete
    // - ref is found and match the new commit
    Ok(())
}

/// Check if the push should be continue, if not return the error message
pub async fn is_legal_push<'a>(
    ref_updates: &[RefUpdatePkt<'a>],
    db: &Arc<dyn NostrDatabase>,
    repo: &PublicKeyAndRepoPath,
) -> Result<HashMap<&'a str, &'static str>, (StatusCode, &'static str)> {
    // Repo announcement from the author
    let Some(repo_announcement) = db
        .query(
            Filter::new()
                .author(repo.public_key)
                .identifier(&repo.repo_name)
                .kind(Kind::GitRepoAnnouncement),
        )
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Database error. Try to push later :(",
            )
        })?
        .first_owned()
    else {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            "No repository announcement found. It should be there but it's not. Broadcast it \
             please :)",
        ));
    };

    // Repo state event from the author or one of the maintainers
    let Some(repo_state) = db
        .query(
            Filter::new()
                .author(repo.public_key)
                .authors(crate::utils::get_maintainers(&repo_announcement).copied())
                .identifier(&repo.repo_name)
                .kind(Kind::RepoState),
        )
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Database error. Try to push later :(",
            )
        })?
        .first_owned()
    else {
        return Err((
            StatusCode::BAD_REQUEST,
            "No repository state announcements found. Broadcast it to the relay first",
        ));
    };

    Ok(ref_updates
        .iter()
        .filter_map(|ref_update| {
            check_ref_update(ref_update, &repo_state)
                .err()
                .map(|err| (ref_update.ref_name, err))
        })
        .collect())
}
