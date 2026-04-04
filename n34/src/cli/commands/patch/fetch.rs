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
    fs,
    path::{Path, PathBuf},
    str::FromStr,
};

use clap::Args;
use nostr::{
    event::{Kind, TagKind},
    filter::Filter,
    nips::nip19::ToBech32,
};

use crate::{
    cli::{
        CliOptions,
        traits::{CommandRunner, OptionNaddrOrSetVecExt, RelayOrSetVecExt},
        types::{NaddrOrSet, NostrEvent},
    },
    error::{N34Error, N34Result},
    nostr_utils::{
        NostrClient,
        traits::{NaddrsUtils, ReposUtils},
        utils,
    },
};

#[derive(Debug, Args)]
pub struct FetchArgs {
    /// Repository address in `naddr` format (`naddr1...`), NIP-05 format
    /// (`4rs.nl/n34` or `_@4rs.nl/n34`), or a set name like `kernel`.
    ///
    /// If omitted, looks for a `nostr-address` file.
    #[arg(value_name = "NADDR-NIP05-OR-SET", long = "repo")]
    naddrs:   Option<Vec<NaddrOrSet>>,
    /// Output directory for the patches. Default to the current directory
    #[arg(short, long, value_name = "PATH")]
    output:   Option<PathBuf>,
    /// The patch id to fetch it
    patch_id: NostrEvent,
}

impl CommandRunner for FetchArgs {
    const NEED_SIGNER: bool = false;

    async fn run(self, options: CliOptions) -> N34Result<()> {
        let naddrs = utils::naddrs_or_file(
            self.naddrs.flat_naddrs(&options.config.sets)?,
            &utils::nostr_address_path()?,
        )?;
        let relays = options.relays.clone().flat_relays(&options.config.sets)?;
        let client = NostrClient::init(&options, &relays).await;
        let output_path = self.output.unwrap_or_default();

        client
            .add_relays(
                &[
                    naddrs.extract_relays(),
                    self.patch_id.relays,
                    client
                        .fetch_repos(&naddrs.into_coordinates())
                        .await?
                        .extract_relays(),
                ]
                .concat(),
            )
            .await;

        let root_patch = client
            .fetch_event(
                Filter::new()
                    .id(self.patch_id.event_id)
                    .kind(Kind::GitPatch),
            )
            .await?
            .ok_or(N34Error::CanNotFoundPatch)?;

        if !root_patch
            .tags
            .iter()
            .any(|t| t.kind() == TagKind::t() && t.content().is_some_and(|c| c == "root"))
        {
            return Err(N34Error::NotRootPatch);
        }

        let root_author = root_patch.pubkey;
        let root_patch = super::GitPatch::from_str(&root_patch.content)
            .map_err(|err| N34Error::InvalidEvent(format!("Failed to parse the patch: {err}")))?;

        tracing::info!("Found the root patch: `{}`", root_patch.subject);

        let mut patches = client
            .fetch_patch_series(self.patch_id.event_id, root_author)
            .await?
            .into_iter()
            .map(|p| {
                let patch = super::GitPatch::from_str(&p.content).map_err(|err| {
                    N34Error::InvalidEvent(format!(
                        "Failed to parse the patch `{}`: {err}",
                        p.id.to_bech32().expect("Infallible")
                    ))
                })?;
                N34Result::Ok((patch.filename(&output_path)?, patch))
            })
            .collect::<N34Result<Vec<_>>>()?;
        patches.push((root_patch.filename(&output_path)?, root_patch));
        patches.sort_unstable_by_key(|p| p.0.clone());
        patches.dedup_by_key(|p| p.0.clone());

        if output_path.as_path() != Path::new("") && !output_path.exists() {
            fs::create_dir_all(&output_path)?;
        }

        for (patch_path, patch) in patches {
            tracing::info!("Writeing `{}` in `{}`", patch.subject, patch_path.display());
            fs::write(patch_path, patch.inner)?;
        }

        Ok(())
    }
}
