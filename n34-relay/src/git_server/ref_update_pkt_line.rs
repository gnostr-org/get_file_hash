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

use crate::utils;

/// The null/zero SHA-1 hash indicating deletion or non-existence.
const NULL_OID: &str = "0000000000000000000000000000000000000000";

/// Represents a ref update pkt-line in a Git protocol.
#[derive(Debug)]
pub struct RefUpdatePkt<'a> {
    /// The old commit hash (before the update).
    pub old_commit:   &'a str,
    /// The new commit hash (after the update).
    pub new_commit:   &'a str,
    /// The name of the reference (e.g., "refs/heads/master").
    pub ref_name:     &'a str,
    /// The capabilities.
    pub capabilities: Option<&'a str>,
}

impl<'a> RefUpdatePkt<'a> {
    /// Parses the ref update payload (without the 4-byte length prefix).
    ///
    /// Expected format: `<old-oid> <new-oid> <ref-name>[\0<capabilities>]`
    pub fn parse(payload: &'a [u8]) -> Result<Self, &'static str> {
        let payload = payload.strip_suffix(b"\n").unwrap_or(payload);

        let str_payload =
            str::from_utf8(payload).map_err(|_| "Invalid pkt-line content: not valid UTF-8")?;
        let mut parts = str_payload.splitn(3, ' ');

        let old_commit = parts.next().ok_or("Missing old commit hash")?;
        let new_commit = parts.next().ok_or("Missing new commit hash")?;
        let ref_with_caps = parts.next().ok_or("Missing ref name")?;

        // pack-protocol: PKT-LINE(command NUL capability-list)
        let (ref_name, capabilities) = match ref_with_caps.split_once('\0') {
            Some((name, caps)) => (name, Some(caps.trim())),
            None => (ref_with_caps, None),
        };

        // Validate OIDs
        if !utils::is_valid_sha1(old_commit) {
            return Err("Invalid old commit hash");
        }
        if !utils::is_valid_sha1(new_commit) {
            return Err("Invalid new commit hash");
        }

        // Validate ref name
        if !ref_name.starts_with("refs/") {
            return Err("Invalid ref name: must start with 'refs/'");
        }

        Ok(Self {
            old_commit,
            new_commit,
            ref_name,
            capabilities,
        })
    }

    /// Returns `true` if this is a branch/ref deletion (new commit is null
    /// OID).
    #[inline]
    #[must_use]
    pub fn is_delete(&self) -> bool {
        self.new_commit == NULL_OID
    }

    /// Returns `true` if this is a new branch/ref creation (old commit is null
    /// OID).
    #[allow(dead_code)] // Used in unit tests
    pub fn is_create(&self) -> bool {
        self.old_commit == NULL_OID
    }
}

/// Parses the 4-byte hex length prefix of a pkt-line.
/// Returns `None` for special packets (flush=0000, delim=0001, etc.)
fn parse_pkt_length(data: &[u8]) -> Result<Option<usize>, &'static str> {
    if data.len() < 4 {
        return Err("Invalid pkt-line: input too short");
    }

    let len_str =
        str::from_utf8(&data[..4]).map_err(|_| "Invalid pkt-line length: not valid UTF-8")?;

    let len = u16::from_str_radix(len_str, 16)
        .map_err(|_| "Invalid pkt-line length: not valid hex")? as usize;

    match len {
        0 => Ok(None), // flush packet (0000)
        1 => Ok(None), // delimiter packet (0001) - skip
        2 => Ok(None), // response-end packet (0002) - skip
        3 => Err("Invalid pkt-line: length too short"),
        _ => Ok(Some(len)),
    }
}

fn collect_pkt_lines(body: &[u8]) -> Result<Vec<&[u8]>, &'static str> {
    let mut pkts = Vec::new();
    let mut pos = 0;

    while pos < body.len() {
        // Need at least 4 bytes for the length prefix
        if body.len() - pos < 4 {
            break;
        }

        // Parse length prefix
        let Some(pkt_len) = parse_pkt_length(&body[pos..])? else {
            break;
        };

        // Validate we have enough data
        if pos + pkt_len > body.len() {
            return Err("Invalid pkt-line: declared length exceeds data length");
        }

        // push the pkt (skip the 4-byte length prefix)
        let pkt = &body[pos + 4..pos + pkt_len];
        pkts.push(pkt);

        pos += pkt_len;
    }

    Ok(pkts)
}

/// Parses pkt-line formatted ref-updates from a byte slice.
///
/// Stops parsing when encountering a flush packet (0000) or end of data.
/// Returns the parsed ref updates
pub fn parse_pkt_lines<'a>(body: &'a [u8]) -> Result<Vec<RefUpdatePkt<'a>>, &'static str> {
    let mut ref_updates = Vec::new();

    // Parse only pkts that looks like command, where the command is:
    //  command =  create / delete / update
    //  create  =  zero-id SP new-id  SP name
    //  delete  =  old-id  SP zero-id SP name
    //  update  =  old-id  SP new-id  SP name
    //
    //  old-id  =  obj-id
    //  new-id  =  obj-id
    //
    // This because we may receive commands and may receive `push-cert` that
    // contains commands `*PKT-LINE(command LF)`. So this parsers will works with
    // both.
    for pkt in collect_pkt_lines(body)? {
        let Ok(pkt_str) = str::from_utf8(pkt) else {
            continue;
        };
        let mut parts = pkt_str.split(' ');

        if utils::is_valid_sha1(parts.next().unwrap_or_default())
            && utils::is_valid_sha1(parts.next().unwrap_or_default())
        {
            // it looks like a command
            ref_updates.push(RefUpdatePkt::parse(pkt)?);
        }
    }

    Ok(ref_updates)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_single_ref_update() {
        let data = b"006bac281124fd463f368106445a4fe4eb251d9c7d7a 4559b8048c334a7e61c76a622cf7cd578a6af406 refs/heads/test2-file";

        let updates = parse_pkt_lines(data).unwrap();
        assert_eq!(updates.len(), 1);
        assert_eq!(updates[0].ref_name, "refs/heads/test2-file");
        assert_eq!(
            updates[0].old_commit,
            "ac281124fd463f368106445a4fe4eb251d9c7d7a"
        );
        assert_eq!(
            updates[0].new_commit,
            "4559b8048c334a7e61c76a622cf7cd578a6af406"
        );
    }

    #[test]
    fn parse_create_branch() {
        let payload = b"0000000000000000000000000000000000000000 53e284c5c3e8b8310077a43d09fd391456f582df refs/heads/new-branch";
        let update = RefUpdatePkt::parse(payload).unwrap();

        assert!(update.is_create());
        assert!(!update.is_delete());
    }

    #[test]
    fn parse_delete_branch() {
        let payload = b"53e284c5c3e8b8310077a43d09fd391456f582df 0000000000000000000000000000000000000000 refs/heads/old-branch";
        let update = RefUpdatePkt::parse(payload).unwrap();

        assert!(update.is_delete());
        assert!(!update.is_create());
    }

    #[test]
    fn flush_stops_parsing() {
        let mut data = Vec::new();
        data.extend_from_slice(b"00b10000000000000000000000000000000000000000 ac281124fd463f368106445a4fe4eb251d9c7d7a refs/heads/master\0report-status-v2 side-band-64k object-format=sha1 agent=git/2.51.2-Linux\n");
        data.extend_from_slice(b"0000"); // Flush
        data.extend_from_slice(b"PACK\x00\x00\x00\x02"); // PACK header

        let updates = parse_pkt_lines(&data).unwrap();
        assert_eq!(updates.len(), 1);

        let update = updates.first().unwrap();
        assert!(update.is_create());
        assert_eq!(
            update.new_commit,
            "ac281124fd463f368106445a4fe4eb251d9c7d7a"
        );
        assert_eq!(update.ref_name, "refs/heads/master");
    }

    #[test]
    fn two_ref_updates() {
        let mut data = Vec::new();
        // first pkt-line
        data.extend_from_slice(b"00B2ac281124fd463f368106445a4fe4eb251d9c7d7a 4559b8048c334a7e61c76a622cf7cd578a6af406 refs/heads/master\0 report-status-v2 side-band-64k object-format=sha1 agent=git/2.51.2-Linux\n");
        // second pkt-line
        data.extend_from_slice(b"006b4559b8048c334a7e61c76a622cf7cd578a6af406 53e284c5c3e8b8310077a43d09fd391456f582df refs/heads/test2-file");
        data.extend_from_slice(b"0000"); // Flush
        data.extend_from_slice(b"PACK\x00\x00\x00\x02"); // PACK header

        let updates = parse_pkt_lines(&data).unwrap();
        assert_eq!(updates.len(), 2);
        let first_update = updates.first().unwrap();
        let second_update = updates.last().unwrap();

        assert_eq!(
            first_update.old_commit,
            "ac281124fd463f368106445a4fe4eb251d9c7d7a"
        );
        assert_eq!(
            first_update.new_commit,
            "4559b8048c334a7e61c76a622cf7cd578a6af406"
        );
        assert_eq!(first_update.ref_name, "refs/heads/master");

        assert_eq!(
            second_update.old_commit,
            "4559b8048c334a7e61c76a622cf7cd578a6af406"
        );
        assert_eq!(
            second_update.new_commit,
            "53e284c5c3e8b8310077a43d09fd391456f582df"
        );
        assert_eq!(second_update.ref_name, "refs/heads/test2-file");
    }
}
