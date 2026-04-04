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

use std::fs;

use nostr::{
    event::{Event, TagKind, TagStandard},
    key::PublicKey,
};

use crate::{
    errors::{RelayError, RelayResult},
    pathes,
};

/// Opens the logs file for writing. If the file size exceeds 5MB, it is opened
/// in write mode, otherwise in append mode.
pub fn logs_file() -> RelayResult<fs::File> {
    const FIVE_MB: u64 = 1024 * 1024 * 5;

    let logs_path = pathes::logs_file_path();
    if let Some(parent) = logs_path.parent()
        && !parent.exists()
    {
        fs::create_dir_all(parent)
            .map_err(|err| RelayError::Fs(parent.to_path_buf(), err.to_string()))?;
    }

    _ = fs::File::create_new(&logs_path);

    let is_large = if let Ok(file) = fs::File::open(&logs_path)
        && let Ok(metadata) = file.metadata()
    {
        metadata.len() >= FIVE_MB
    } else {
        false
    };

    fs::OpenOptions::new()
        .write(true)
        .append(!is_large)
        .truncate(is_large)
        .open(&logs_path)
        .map_err(|err| RelayError::Fs(logs_path, err.to_string()))
}

/// Replaces template variables in content with their value.
fn homepage_variables(content: &str) -> String {
    content.replace("{{VERSION}}", env!("CARGO_PKG_VERSION"))
}

/// Returns the content of the homepage if the file exists, otherwise the
/// default homepage
pub fn homepage_content() -> String {
    let default_page = include_str!("default-homepage.html");
    let homepage_path = pathes::homepage_file_path();

    if !homepage_path.exists() {
        tracing::debug!(
            "No custom home page in `{}`, using the default one",
            homepage_path.display()
        );
        return homepage_variables(default_page);
    }

    match fs::read_to_string(&homepage_path) {
        Ok(content) => {
            tracing::info!("Using the custom home page: `{}`", homepage_path.display());
            homepage_variables(&content)
        }
        Err(err) => {
            tracing::error!(
                "Failed to get the custom home page content from `{}`: {err}",
                homepage_path.display()
            );
            homepage_variables(default_page)
        }
    }
}

/// Removes the protocol part (e.g. `http://`, `https://`) from a URL string.
///
/// If no protocol is found, returns the original string unchanged.
pub fn remove_proto(str_url: &str) -> &str {
    const SEPARATOR: &str = "://";

    if let Some(index) = str_url.find(SEPARATOR) {
        &str_url[index + SEPARATOR.len()..]
    } else {
        str_url
    }
}

/// Fetches the maintainers associated with the event. Returns an empty iterator
/// if no maintainers are found.
#[inline]
pub fn get_maintainers(event: &Event) -> impl Iterator<Item = &PublicKey> {
    event
        .tags
        .find_standardized(TagKind::Maintainers)
        .map(|tag| {
            if let TagStandard::GitMaintainers(pubkeys) = tag {
                return pubkeys.as_slice();
            }
            unreachable!("TagKind::Maintainers")
        })
        .unwrap_or(&[])
        .iter()
}

/// Validates that a string is a valid 40-character hex SHA-1 OID.
#[inline]
pub fn is_valid_sha1(s: &str) -> bool {
    s.len() == 40 && s.chars().all(|c| c.is_ascii_hexdigit())
}
