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

use std::net::{IpAddr, Ipv4Addr};
use std::path::PathBuf;

use parking_lot::RwLock;

use crate::ext_traits::RwlockVecExt;
use crate::pathes;

/// `NetworkConfig` defaults
pub mod net {
    use super::*;

    #[inline]
    pub const fn ip_addr() -> IpAddr {
        IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1))
    }

    #[inline]
    pub const fn port() -> u16 {
        3598
    }
}

/// `LmdbConfig` defaults
pub mod lmdb {
    use super::*;

    #[inline]
    pub fn dir() -> PathBuf {
        pathes::lmdb_dir_path()
    }

    #[inline]
    pub const fn map_size() -> usize {
        if cfg!(target_pointer_width = "32") {
            1024 * 1024 * 1024 * 4
        } else {
            1024 * 1024 * 1024 * 32
        }
    }

    #[inline]
    pub const fn max_readers() -> u32 {
        126
    }

    #[inline]
    pub const fn additional_dbs() -> u32 {
        0
    }
}

/// `RatelimitConfig` defaults
pub mod ratelimit {
    #[inline]
    pub const fn max_queries() -> usize {
        500
    }

    #[inline]
    pub const fn events_per_minute() -> u32 {
        120
    }
}

/// `RhaiPluginsConfig`
pub mod rhai {
    use std::num::NonZeroU8;

    pub const fn workers() -> NonZeroU8 {
        unsafe { NonZeroU8::new_unchecked(3) }
    }
}

pub mod grasp {
    use std::borrow::Cow;

    pub const fn enable() -> bool {
        true
    }

    pub const fn git_path() -> Cow<'static, str> {
        Cow::Borrowed("git")
    }
}

/// `Nip11Config` defaults
pub mod nip11 {
    #[inline]
    pub const fn supported_nips() -> &'static [u16] {
        &[1, 9, 13, 17, 40, 42, 50, 59, 62, 70, 77]
    }

    #[inline]
    pub const fn supported_grasps() -> &'static [&'static str] {
        &["GRASP-01"]
    }

    #[inline]
    pub const fn software() -> &'static str {
        "https://relay.n34.dev"
    }

    #[inline]
    pub const fn version() -> &'static str {
        // e.g. `0.1.0`
        env!("CARGO_PKG_VERSION")
    }
}

/// `RelayConfig` defaults
pub mod relay {
    use std::num::NonZeroUsize;

    #[inline]
    pub const fn nip42() -> bool {
        false
    }

    #[inline]
    pub const fn min_pow() -> u8 {
        0
    }

    #[inline]
    pub const fn max_event_size() -> NonZeroUsize {
        unsafe { NonZeroUsize::new_unchecked(1024 * 150) }
    }

    #[inline]
    pub const fn max_limit() -> NonZeroUsize {
        unsafe { NonZeroUsize::new_unchecked(5000) }
    }
    #[inline]
    pub const fn default_limit() -> NonZeroUsize {
        unsafe { NonZeroUsize::new_unchecked(500) }
    }
    #[inline]
    pub const fn max_subid_length() -> NonZeroUsize {
        unsafe { NonZeroUsize::new_unchecked(150) }
    }
}

impl Default for super::NetworkConfig {
    fn default() -> Self {
        use self::net::*;

        Self {
            ip:   ip_addr(),
            port: port(),
        }
    }
}

impl Default for super::LmdbConfig {
    fn default() -> Self {
        use self::lmdb::*;

        Self {
            dir:            dir(),
            map_size:       map_size(),
            max_readers:    max_readers(),
            additional_dbs: additional_dbs(),
        }
    }
}

impl Default for super::RatelimitConfig {
    fn default() -> Self {
        use self::ratelimit::*;

        Self {
            max_queries:       max_queries(),
            events_per_minute: events_per_minute(),
        }
    }
}

impl Default for super::RhaiPluginsConfig {
    fn default() -> Self {
        use self::rhai::*;

        Self {
            workers: workers(),
            plugins: Vec::new(),
        }
    }
}

impl Default for super::GraspConfig {
    fn default() -> Self {
        use self::grasp::*;

        Self {
            enable:      true,
            git_path:    git_path(),
            max_reqs:    None,
            req_timeout: None,
            repos_path:  pathes::grasp_repos(),
        }
    }
}

impl Default for super::Nip11Config {
    fn default() -> Self {
        use self::nip11::*;

        Self {
            name:             RwLock::new(None),
            description:      RwLock::new(None),
            banner:           RwLock::new(None),
            icon:             RwLock::new(None),
            admin:            Option::None,
            contact:          Option::None,
            privacy_policy:   Option::None,
            terms_of_service: Option::None,
            limitation:       super::Nip11Limitation::default(),

            supported_nips:   supported_nips(),
            supported_grasps: supported_grasps(),
            software:         software(),
            version:          version(),
        }
    }
}

impl Default for super::CoreConfig {
    fn default() -> Self {
        use relay::*;

        Self {
            domain:           String::new(),
            nip42:            nip42(),
            max_connections:  None,
            min_pow:          min_pow(),
            max_event_size:   max_event_size(),
            max_limit:        max_limit(),
            default_limit:    default_limit(),
            max_subid_length: max_subid_length(),
            whitelist:        RwlockVecExt::new_empty(),
            blacklist:        RwlockVecExt::new_empty(),
            admins:           RwlockVecExt::new_empty(),
            allowed_kinds:    RwlockVecExt::new_empty(),
            disallowed_kinds: RwlockVecExt::new_empty(),
        }
    }
}
