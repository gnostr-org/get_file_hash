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

use nostr::{event::EventId, nips::nip19::Nip19Coordinate, types::RelayUrl};

use super::CliOptions;
use crate::{
    cli::{
        ConfigError,
        RepoRelaySet,
        types::{NaddrOrSet, NostrEvent, RelayOrSet},
    },
    error::{N34Error, N34Result},
};

/// A trait defining the interface for command runners in the CLI.
pub trait CommandRunner {
    /// Whether this command needs the relays option (false by default).
    /// Only applies to commands, not subcommands.
    const NEED_RELAYS: bool = false;
    /// Indicates if this command requires the signer. Defaults to true.
    /// Only applies to commands, not subcommands.
    const NEED_SIGNER: bool = true;

    /// Executes the command and returns a Result indicating success or failure.
    fn run(self, options: CliOptions) -> impl Future<Output = N34Result<()>> + Send;
}

#[easy_ext::ext(VecNostrEventExt)]
impl Vec<NostrEvent> {
    /// Extracts `EventId` from each `NostrEvent` and collects them into a
    /// `Vec<EventId>`.
    pub fn into_event_ids(self) -> Vec<EventId> {
        self.into_iter().map(|e| e.event_id).collect()
    }
}

#[easy_ext::ext(NaddrOrSetVecExt)]
impl Vec<NaddrOrSet> {
    /// Converts this vector of [`NaddrOrSet`] into a flat vector of
    /// [`Nip19Coordinate`] using the given sets.
    pub fn flat_naddrs(self, sets: &[RepoRelaySet]) -> N34Result<Vec<Nip19Coordinate>> {
        self.into_iter()
            .map(|n| n.get_naddrs(sets))
            .try_fold(Vec::new(), |mut acc, item| {
                acc.extend(item?);
                Ok(acc)
            })
    }
}

#[easy_ext::ext(RelayOrSetVecExt)]
impl Vec<RelayOrSet> {
    /// Converts this vector of [`RelayOrSet`] into a flat vector of
    /// [`RelayUrl`] using the given sets.
    pub fn flat_relays(self, sets: &[RepoRelaySet]) -> N34Result<Vec<RelayUrl>> {
        self.into_iter()
            .map(|n| n.get_relays(sets))
            .try_fold(Vec::new(), |mut acc, item| {
                acc.extend(item?);
                Ok(acc)
            })
    }
}


#[easy_ext::ext(OptionNaddrOrSetVecExt)]
impl Option<Vec<NaddrOrSet>> {
    /// Converts this vector of [`NaddrOrSet`] into a flat vector of
    /// [`Nip19Coordinate`] using the given sets.
    pub fn flat_naddrs(&self, sets: &[RepoRelaySet]) -> N34Result<Option<Vec<Nip19Coordinate>>> {
        // Clones self here to simplify command code
        self.clone()
            .map(|naddrs| naddrs.flat_naddrs(sets))
            .transpose()
    }
}

#[easy_ext::ext(MutRepoRelaySetsExt)]
impl Vec<RepoRelaySet> {
    /// Removes duplicate repository addresses from each set.
    ///
    /// Relays are automatically deduplicated by the HashSet, but
    /// repository addresses may appear duplicated if relays are sorted
    /// differently or when relay counts vary. This compares addresses by
    /// their coordinates, ignoring any embedded relay details.
    pub fn dedup_naddrs(&mut self) {
        self.iter_mut().for_each(RepoRelaySet::dedup_naddrs);
    }

    /// Finds and returns a mutable reference a set with the given name. Returns
    /// an error if no set with this name exists.
    pub fn get_mut_set(&mut self, name: impl AsRef<str>) -> N34Result<&mut RepoRelaySet> {
        let name = name.as_ref();
        let set = self
            .iter_mut()
            .find(|set| set.name == name)
            .ok_or_else(|| N34Error::from(ConfigError::SetNotFound(name.to_owned())))?;

        tracing::trace!(
            name = %name, set = ?set,
            "Successfully located a set with the giving name"
        );

        Ok(set)
    }

    /// Creates and pushes a new set with the given name.
    ///
    /// Returns an error if a set with the same name already exists.
    pub fn push_set(
        &mut self,
        name: impl Into<String>,
        repos: impl IntoIterator<Item = Nip19Coordinate>,
        relays: impl IntoIterator<Item = RelayUrl>,
    ) -> N34Result<()> {
        let set_name: String = name.into();
        tracing::trace!(sets = ?self, "Pushing set '{set_name}' to sets collection");

        if self.as_slice().exists(&set_name) {
            return Err(ConfigError::SetDuplicateName(set_name).into());
        }

        self.push(RepoRelaySet::new(set_name, repos, relays));

        Ok(())
    }

    /// Removes the set with the given name if it exists. Returns an error if
    /// the set is not found.
    pub fn remove_set(&mut self, name: impl Into<String>) -> N34Result<()> {
        let set_name: String = name.into();
        tracing::trace!(set_name, sets = ?self, "Removing set '{set_name}' from sets collection");

        if !self.as_slice().exists(&set_name) {
            return Err(ConfigError::SetNotFound(set_name).into());
        }

        self.retain(|s| s.name != set_name);

        Ok(())
    }

    /// Removes the given relays from the specified set.
    pub fn remove_relays(
        &mut self,
        name: impl Into<String>,
        relays: impl Iterator<Item = RelayUrl>,
    ) -> N34Result<()> {
        let relays = Vec::from_iter(relays);
        let set = self.get_mut_set(name.into())?;

        set.relays.retain(|r| !relays.contains(r));

        Ok(())
    }

    /// Removes the given naddrs from the specified set.
    pub fn remove_naddrs(
        &mut self,
        name: impl Into<String>,
        naddrs: impl Iterator<Item = Nip19Coordinate>,
    ) -> N34Result<()> {
        let coordinates = Vec::from_iter(naddrs.map(|n| n.coordinate));
        let set = self.get_mut_set(name.into())?;

        set.naddrs.retain(|n| !coordinates.contains(&n.coordinate));

        Ok(())
    }
}

#[easy_ext::ext(RepoRelaySetsExt)]
impl &[RepoRelaySet] {
    /// Checks for duplicate set names. Returns an error if any duplicates are
    /// found.
    pub fn ensure_names(&self) -> N34Result<()> {
        let mut names = Vec::with_capacity(self.len());
        names.extend(self.iter().map(|s| s.name.to_owned()));

        names.sort_unstable();

        if let Some(duplicate) = duplicate_in_sorted(&names) {
            return Err(ConfigError::SetDuplicateName(duplicate.clone()).into());
        }
        Ok(())
    }

    /// Check if a set with the given name exists.
    pub fn exists(&self, set_name: &str) -> bool {
        self.iter().any(|set| set.name == set_name)
    }

    /// Finds and returns a reference a set with the given name. Returns an
    /// error if no set with this name exists.
    pub fn get_set(&self, name: impl AsRef<str>) -> N34Result<&RepoRelaySet> {
        let name = name.as_ref();
        let set = self
            .iter()
            .find(|set| set.name == name)
            .ok_or_else(|| N34Error::from(ConfigError::SetNotFound(name.to_owned())))?;
        tracing::trace!(
            name = %name, set = ?set,
            "Successfully located a set with the giving name"
        );
        Ok(set)
    }
}

/// Helper function that checks for duplicates in a sorted slice
fn duplicate_in_sorted<T: PartialEq + Clone>(items: &[T]) -> Option<&T> {
    items.windows(2).find(|w| w[0] == w[1]).map(|w| &w[0])
}
