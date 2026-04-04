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

/// `issue close` subcommand
mod close;
/// `issue list` subcommand
mod list;
/// `issue new` subcommand
mod new;
/// `issue reopen` subcommand
mod reopen;
/// `issue resolve` subcommand
mod resolve;
/// `issue view` subcommand
mod view;

use std::fmt;

use clap::Subcommand;
use nostr::event::Kind;

use self::close::CloseArgs;
use self::list::ListArgs;
use self::new::NewArgs;
use self::reopen::ReopenArgs;
use self::resolve::ResolveArgs;
use self::view::ViewArgs;
use super::{CliOptions, CommandRunner};
use crate::error::{N34Error, N34Result};

/// Prefix used for git issue alt.
pub const ISSUE_ALT_PREFIX: &str = "git issue: ";

#[derive(Subcommand, Debug)]
pub enum IssueSubcommands {
    /// Create a new repository issue
    New(NewArgs),
    /// View an issue by its ID
    View(ViewArgs),
    /// Reopens a closed issue.
    Reopen(ReopenArgs),
    /// Closes an open issue.
    Close(CloseArgs),
    /// Resolves an issue.
    Resolve(ResolveArgs),
    /// List the repositories issues.
    List(ListArgs),
}

/// Possible states for a Git issue
#[derive(Debug)]
pub enum IssueStatus {
    /// The issue is currently open
    Open,
    /// The issue has been resolved
    Resolved,
    /// The issue has been closed
    Closed,
}

impl IssueStatus {
    /// Maps the issue status to its corresponding Nostr kind.
    #[inline]
    pub fn kind(&self) -> Kind {
        match self {
            Self::Open => Kind::GitStatusOpen,
            Self::Resolved => Kind::GitStatusApplied,
            Self::Closed => Kind::GitStatusClosed,
        }
    }

    /// Returns the string representation of the issue status.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Open => "Open",
            Self::Resolved => "Resolved",
            Self::Closed => "Closed",
        }
    }

    /// Check if the issue is open.
    #[inline]
    pub fn is_open(&self) -> bool {
        matches!(self, Self::Open)
    }

    /// Check if the issue is resolved.
    #[inline]
    pub fn is_resolved(&self) -> bool {
        matches!(self, Self::Resolved)
    }

    /// Check if the issue is closed.
    #[inline]
    pub fn is_closed(&self) -> bool {
        matches!(self, Self::Closed)
    }
}

impl From<&IssueStatus> for Kind {
    fn from(status: &IssueStatus) -> Self {
        status.kind()
    }
}

impl fmt::Display for IssueStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl TryFrom<Kind> for IssueStatus {
    type Error = N34Error;

    fn try_from(kind: Kind) -> Result<Self, Self::Error> {
        match kind {
            Kind::GitStatusOpen => Ok(Self::Open),
            Kind::GitStatusApplied => Ok(Self::Resolved),
            Kind::GitStatusClosed => Ok(Self::Closed),
            _ => Err(N34Error::InvalidIssueStatus(kind)),
        }
    }
}

impl CommandRunner for IssueSubcommands {
    async fn run(self, options: CliOptions) -> N34Result<()> {
        crate::run_command!(self, options, & New View Reopen Close Resolve List)
    }
}
