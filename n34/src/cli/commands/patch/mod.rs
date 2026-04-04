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

/// `patch apply` suubcommand
mod apply;
/// `patch close` subcommand
mod close;
/// `patch draft` subcommand
mod draft;
/// `patch fetch` subcommand
mod fetch;
/// `patch list` subcommand
mod list;
/// `patch merge` subcommand
mod merge;
/// `patch reopen` subcommand
mod reopen;
/// `patch send` subcommand
mod send;
#[cfg(test)]
mod tests;

use std::{
    fmt,
    path::{Path, PathBuf},
    str::FromStr,
    sync::LazyLock,
};

use clap::Subcommand;
use nostr::event::Kind;
use regex::Regex;

use self::apply::ApplyArgs;
use self::close::CloseArgs;
use self::draft::DraftArgs;
use self::fetch::FetchArgs;
use self::list::ListArgs;
use self::merge::MergeArgs;
use self::reopen::ReopenArgs;
use self::send::SendArgs;
use super::{CliOptions, CommandRunner};
use crate::error::{N34Error, N34Result};

/// Regular expression for extracting the patch subject.
static SUBJECT_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?m)^Subject: (.*(?:\n .*)*)").unwrap());

/// Regular expression for extracting the patch body.
static BODY_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\n\n((?:.|\n)*?)(?:\n--[ -]|\z)").unwrap());

/// Regular expiration for extracting the patch version and number
static PATCH_VERSION_NUMBER_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\[PATCH\s+(?:v(?<version>\d+)\s*)?(?<number>\d+)/(?:\d+)").unwrap()
});

/// Content of the hashtag representing the root patch.
pub const ROOT_HASHTAG_CONTENT: &str = "root";
/// Content of the hashtag representing the root revision patch.
pub const REVISION_ROOT_HASHTAG_CONTENT: &str = "root-revision";
/// The content of the hashtag used by `ngit-cli` to represent a root revision
/// patch before the commit 6ae42e67d9da36f6c2e1356acba30a3a62112bc7. This was a
/// typo.
pub const LEGACY_NGIT_REVISION_ROOT_HASHTAG_CONTENT: &str = "revision-root";

#[derive(Subcommand, Debug)]
pub enum PatchSubcommands {
    /// Send one or more patches to a repository.
    Send(SendArgs),
    /// Fetches a patch by its id.
    Fetch(FetchArgs),
    /// Closes an open or drafted patch.
    Close(CloseArgs),
    /// Converts an open patch to draft state.
    Draft(DraftArgs),
    /// Reopens a closed or drafted patch.
    Reopen(ReopenArgs),
    /// Set an open patch status to applied.
    Apply(ApplyArgs),
    /// Set an open patch status to merged.
    Merge(MergeArgs),
    /// List the repositories patches.
    List(ListArgs),
}

/// Represents a git patch
#[derive(Clone, Debug)]
pub struct GitPatch {
    /// Full content of the patch file
    pub inner:   String,
    /// Short description of the patch changes
    pub subject: String,
    /// Detailed explanation of the patch changes
    pub body:    String,
}

#[derive(Debug)]
pub enum PatchStatus {
    /// The patch is currently open
    Open,
    /// The patch has been merged/applied
    MergedApplied,
    /// The patch has been closed
    Closed,
    /// A patch that has been drafted but not yet applied.
    Draft,
}

impl PatchStatus {
    /// Maps the issue status to its corresponding Nostr kind.
    #[inline]
    pub fn kind(&self) -> Kind {
        match self {
            Self::Open => Kind::GitStatusOpen,
            Self::MergedApplied => Kind::GitStatusApplied,
            Self::Closed => Kind::GitStatusClosed,
            Self::Draft => Kind::GitStatusDraft,
        }
    }

    /// Returns the string representation of the patch status.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Open => "Open",
            Self::MergedApplied => "Merged/Applied",
            Self::Closed => "Closed",
            Self::Draft => "Draft",
        }
    }

    /// Check if the patch is open.
    #[inline]
    pub fn is_open(&self) -> bool {
        matches!(self, Self::Open)
    }

    /// Check if the patch is merged/applied.
    #[inline]
    pub fn is_merged_or_applied(&self) -> bool {
        matches!(self, Self::MergedApplied)
    }

    /// Check if the patch is closed.
    #[inline]
    pub fn is_closed(&self) -> bool {
        matches!(self, Self::Closed)
    }

    /// Check if the patch is drafted
    #[inline]
    pub fn is_drafted(&self) -> bool {
        matches!(self, Self::Draft)
    }
}

impl From<&PatchStatus> for Kind {
    fn from(status: &PatchStatus) -> Self {
        status.kind()
    }
}

impl fmt::Display for PatchStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}


impl TryFrom<Kind> for PatchStatus {
    type Error = N34Error;

    fn try_from(kind: Kind) -> Result<Self, Self::Error> {
        match kind {
            Kind::GitStatusOpen => Ok(Self::Open),
            Kind::GitStatusApplied => Ok(Self::MergedApplied),
            Kind::GitStatusClosed => Ok(Self::Closed),
            Kind::GitStatusDraft => Ok(Self::Draft),
            _ => Err(N34Error::InvalidPatchStatus(kind)),
        }
    }
}

impl GitPatch {
    /// Returns the patch file name from the subject
    pub fn filename(&self, parent: impl AsRef<Path>) -> N34Result<PathBuf> {
        let (patch_version, patch_number) = if self.subject.contains("[PATCH]") {
            (String::new(), "1")
        } else {
            patch_version_and_subject(&self.subject)?
        };

        let patch_name = if patch_number == "0" {
            "cover-letter".to_owned()
        } else {
            patch_file_name(&self.subject)?
        };

        Ok(parent
            .as_ref()
            .join(format!("{patch_version}{patch_number:0>4}-{patch_name}").replace("--", "-"))
            .with_extension("patch"))
    }
}

impl CommandRunner for PatchSubcommands {
    async fn run(self, options: CliOptions) -> N34Result<()> {
        crate::run_command!(self, options, & Send Fetch Close Reopen Draft Apply Merge List)
    }
}

impl FromStr for GitPatch {
    type Err = String;

    fn from_str(patch_content: &str) -> Result<Self, Self::Err> {
        // Regex for subject (handles multi-line subjects)
        let subject = SUBJECT_RE
            .captures(patch_content)
            .and_then(|cap| cap.get(1))
            .ok_or("No subject found")?
            .as_str()
            .trim()
            .replace('\n', "")
            .to_string();

        // Regex for body
        let body = BODY_RE
            .captures(patch_content)
            .and_then(|cap| cap.get(1))
            .ok_or("No body found")?
            .as_str()
            .trim()
            .to_string();
        Ok(Self {
            inner: patch_content.to_owned(),
            subject,
            body,
        })
    }
}

/// Extracts the version prefix and patch number from a patch subject string.
///
/// The version prefix is formatted as "v{version}-" if present, or an empty
/// string. The patch number is mandatory and will cause an error if not found.
fn patch_version_and_subject(subject: &str) -> N34Result<(String, &str)> {
    let captures = PATCH_VERSION_NUMBER_RE.captures(subject).ok_or_else(|| {
        N34Error::InvalidEvent(format!("Can not parse the patch subject `{subject}`"))
    })?;
    Ok((
        captures
            .name("version")
            .map(|m| format!("v{}-", m.as_str()))
            .unwrap_or_default(),
        captures
            .name("number")
            .map(|m| m.as_str())
            .expect("It's not optional, regex will fail if it's not found"),
    ))
}

/// Extracts a clean file name from the patch subject by removing version info
/// and special characters. Converts to lowercase and ensures the name only
/// contains alphanumeric, '.', '-', or '_' characters.
fn patch_file_name(subject: &str) -> N34Result<String> {
    Ok(subject
        .split_once("]")
        .ok_or_else(|| {
            N34Error::InvalidEvent(format!(
                "Invalid patch subject. No `[PATCH ...]`: `{subject}`",
            ))
        })?
        .1
        .trim()
        .to_lowercase()
        .replace(
            |c: char| !c.is_ascii_alphanumeric() && !['.', '-', '_'].contains(&c),
            "-",
        )
        .chars()
        .take(60)
        .collect::<String>()
        .trim_matches('-')
        .trim()
        .replace("--", "-"))
}
