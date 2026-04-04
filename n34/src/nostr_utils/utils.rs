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

use std::{
    fmt,
    fs,
    path::{Path, PathBuf},
    str::FromStr,
    sync::atomic::Ordering,
};

use nostr::{
    event::{Event, EventId, Kind, Tag, TagKind, TagStandard},
    filter::Alphabet,
    key::PublicKey,
    nips::{
        nip01::Coordinate,
        nip10::Marker,
        nip19::{Nip19Coordinate, Nip19Event, ToBech32},
        nip34::GitRepositoryAnnouncement,
        nip65::{self, RelayMetadata},
    },
    types::RelayUrl,
};

use super::traits::TagsExt;
use crate::{
    cli::{NOSTR_ADDRESS_FILE, parsers},
    error::{N34Error, N34Result},
};

/// Returns the value of the given tag
#[inline]
fn tag_value(tag: &TagStandard) -> String {
    tag.clone().to_vec().remove(1)
}

/// Parses the tag value into type `T` if possible.
#[inline]
fn parse_value<T: FromStr>(tag: &TagStandard) -> Option<T> {
    tag_value(tag).parse().ok()
}

/// Gets all values from the tag. If any value fails to parse, returns an empty
/// vector.
#[inline]
fn tag_values<T>(tag: &TagStandard) -> Vec<T>
where
    T: FromStr + fmt::Debug,
    <T as FromStr>::Err: fmt::Debug,
{
    tag.clone()
        .to_vec()
        .into_iter()
        .skip(1)
        .map(|t| {
            let result = T::from_str(t.as_str());
            tracing::trace!("Parsing `{t}` result: `{result:?}`");
            result
        })
        .collect::<Result<_, _>>()
        .unwrap_or_default()
}

/// Convert [`Event`] to [`GitRepositoryAnnouncement`]
pub fn event_into_repo(event: Event, repo_id: impl Into<String>) -> GitRepositoryAnnouncement {
    let tags = &event.tags;

    GitRepositoryAnnouncement {
        id:          repo_id.into(),
        name:        tags.map_tag(TagKind::Name, tag_value),
        description: tags.map_tag(TagKind::Description, tag_value),
        euc:         tags
            .map_marker(
                TagKind::single_letter(Alphabet::R, false),
                "euc",
                parse_value,
            )
            .flatten(),
        web:         tags.dmap_tag(TagKind::Web, tag_values),
        clone:       tags.dmap_tag(TagKind::Clone, tag_values),
        relays:      tags.dmap_tag(TagKind::Relays, tag_values),
        maintainers: tags.dmap_tag(TagKind::Maintainers, tag_values),
    }
}

/// Returns a new string with leading and trailing whitespace removed.
pub fn str_trim(s: String) -> String {
    s.trim().to_owned()
}

/// Returns a vector with duplicate elements removed.
pub fn dedup<I, T>(iter: I) -> Vec<T>
where
    T: std::cmp::Ord,
    I: Iterator<Item = T>,
{
    let mut vector: Vec<T> = iter.collect();
    vector.sort_unstable();
    vector.dedup();
    vector
}

/// Sorts items from the iterator using the given key function.
/// The sorting is unstable, but faster than stable sorting.
pub fn sort_by_key<I, T, K>(iterator: I, key: impl FnMut(&T) -> K) -> impl Iterator<Item = T>
where
    I: IntoIterator<Item = T>,
    K: Ord,
{
    let mut vector = Vec::<T>::from_iter(iterator);
    vector.sort_unstable_by_key(key);
    vector.into_iter()
}

/// Creates a new NIP-19 nevent string from an event ID and up to 3 unique relay
/// URLs.
#[inline]
pub fn new_nevent(event_id: EventId, relays: &[RelayUrl]) -> N34Result<String> {
    Nip19Event::new(event_id)
        .relays(
            dedup(relays.iter().cloned())
                .into_iter()
                .take(3)
                .collect::<Vec<_>>(),
        )
        .to_bech32()
        .map_err(N34Error::from)
}

/// Creates a NIP-19 naddr string for a git repository announcement and up to 3
/// unique relay URLs.
#[inline]
pub fn repo_naddr(
    repo_id: impl Into<String>,
    pubk: PublicKey,
    relays: &[RelayUrl],
) -> N34Result<String> {
    Nip19Coordinate::new(
        Coordinate::new(Kind::GitRepoAnnouncement, pubk).identifier(repo_id),
        dedup(relays.iter().cloned()).into_iter().take(3),
    )
    .to_bech32()
    .map_err(N34Error::from)
}

/// Extracts write relay URLs from an event if present, otherwise returns an
/// empty vector.
pub fn add_write_relays(event: Option<&Event>) -> Vec<RelayUrl> {
    let mut vector = Vec::new();
    if let Some(event) = event {
        vector.extend(
            nip65::extract_owned_relay_list(event.clone())
                .filter_map(|(r, m)| m.is_none_or(|m| m == RelayMetadata::Write).then_some(r)),
        );
    }
    vector
}

/// Extracts read relay URLs from an event if present, otherwise returns an
/// empty vector.
pub fn add_read_relays(event: Option<&Event>) -> Vec<RelayUrl> {
    let mut vector = Vec::new();
    if let Some(event) = event {
        vector.extend(
            nip65::extract_owned_relay_list(event.clone())
                .filter_map(|(r, m)| m.is_none_or(|m| m == RelayMetadata::Read).then_some(r)),
        );
    }
    vector
}


/// Opens the user's default editor ($EDITOR) to edit a temporary file with
/// given suffix, then reads and returns the file contents. The temporary file
/// is automatically deleted.
pub fn read_editor(file_pre_content: Option<&str>, file_suffix: &str) -> N34Result<String> {
    let Ok(editor) = std::env::var("EDITOR") else {
        return Err(N34Error::EditorNotFound);
    };

    let temp_path = tempfile::NamedTempFile::with_suffix(file_suffix)?.into_temp_path();

    if let Some(pre_content) = file_pre_content {
        fs::write(&temp_path, pre_content)?;
    }

    // Disable the logs to not show up in a terminal text editor
    crate::EDITOR_OPEN.store(true, Ordering::Relaxed);
    let exit_status = std::process::Command::new(&editor)
        .arg(temp_path.to_str().expect("The path is valid utf8"))
        .spawn()?
        .wait()?;
    crate::EDITOR_OPEN.store(false, Ordering::Relaxed);

    if !exit_status.success()
        && let Some(code) = exit_status.code()
    {
        return Err(N34Error::EditorErr(editor, code));
    }

    let content = fs::read_to_string(&temp_path)
        .map_err(N34Error::from)?
        .trim()
        .to_owned();

    if content.is_empty() {
        return Err(N34Error::EmptyEditorFile);
    }
    Ok(content)
}

/// Returns the given content if it's `Option::Some` or call [`read_editor`]
pub fn get_content(
    content: Option<impl AsRef<str>>,
    quoted_content: Option<impl AsRef<str>>,
    file_suffix: &str,
) -> N34Result<String> {
    if let Some(content) = content {
        return Ok(content.as_ref().trim().to_owned());
    }
    read_editor(
        quoted_content.map(|s| s.as_ref().to_owned()).as_deref(),
        file_suffix,
    )
}

/// Path to the `nostr-address` file in current directory.
#[inline]
pub fn nostr_address_path() -> std::io::Result<PathBuf> {
    std::env::current_dir().map(|p| p.join(NOSTR_ADDRESS_FILE))
}

/// Returns the given coordinates if Some, otherwise attempts to read and parse
/// coordinates from the specified file. Returns an empty vector if the file
/// doesn't exist.
pub fn naddrs_or_file(
    naddrs: Option<Vec<Nip19Coordinate>>,
    address_file_path: &Path,
) -> N34Result<Vec<Nip19Coordinate>> {
    if let Some(naddrs) = naddrs {
        return Ok(naddrs);
    }

    if address_file_path.exists() {
        parsers::parse_nostr_address_file(address_file_path)
    } else {
        Ok(Vec::new())
    }
}

/// Generate a reply tag for an event with the given ID, relay URL (if any), and
/// marker.
#[inline]
pub fn event_reply_tag(reply_to: &EventId, relay: Option<&RelayUrl>, marker: Marker) -> Tag {
    Tag::custom(
        TagKind::e(),
        [
            reply_to.to_hex(),
            relay.map(|r| r.to_string()).unwrap_or_default(),
            marker.to_string(),
        ],
    )
}

/// Wraps text into lines no longer than max_width, breaking only at whitespace.
pub fn smart_wrap(text: &str, max_width: usize) -> String {
    text.lines()
        .map(|line| {
            if !line.trim().is_empty() {
                line.split(" ")
                    .fold((String::new(), 0), |(result, last_newline), word| {
                        let result_len = result.chars().count();
                        if result_len == 0 {
                            (word.to_owned(), 0)
                        } else if (result_len - last_newline) + word.chars().count() > max_width {
                            (format!("{result}\n{word}"), result_len + 1)
                        } else {
                            (format!("{result} {word}"), last_newline)
                        }
                    })
                    .0
            } else {
                String::new()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Returns an error if the given naddrs is empty otherwise returned it
pub fn check_empty_naddrs(naddrs: Vec<Nip19Coordinate>) -> N34Result<Vec<Nip19Coordinate>> {
    if naddrs.is_empty() {
        return Err(N34Error::EmptyNaddrs);
    }

    Ok(naddrs)
}
