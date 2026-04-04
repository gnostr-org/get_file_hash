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

use std::path::PathBuf;
use std::env;

const BASE_DIR_ENV_VAR: &str = "N34_RELAY_BASE_DIR";

/// Returns the base directory for n34-relay.
/// Priority:
/// 1. Environment variable `N34_RELAY_BASE_DIR`
/// 2. OS-specific config directory + `/n34-relay`
/// 3. Fallback to current directory if config_dir is unavailable
pub fn get_base_dir() -> PathBuf {
    if let Ok(env_path) = env::var(BASE_DIR_ENV_VAR) {
        return PathBuf::from(env_path);
    }

    // Linux:   /home/alice/.config/n34-relay
    // macOS:   /Users/alice/Library/Application Support/n34-relay
    // Windows: C:\Users\Alice\AppData\Roaming\n34-relay
    dirs::config_dir()
        .map(|path| path.join("n34-relay"))
        .unwrap_or_else(|| PathBuf::from("."))
}

/// Alias for get_base_dir to fix E0425 errors in downstream functions
pub fn base_dir_path() -> PathBuf {
    get_base_dir()
}

/// Helper for the default config file path
pub fn get_default_config_path() -> PathBuf {
    base_dir_path().join("config.toml")
}

pub fn config_file_path() -> PathBuf {
    get_default_config_path()
}

pub fn lmdb_dir_path() -> PathBuf {
    base_dir_path().join("lmdb")
}

pub fn logs_file_path() -> PathBuf {
    base_dir_path().join("logs.log")
}

pub fn homepage_file_path() -> PathBuf {
    base_dir_path().join("homepage.html")
}

pub fn rhai_plugins_dir() -> PathBuf {
    base_dir_path().join("rhai-plugins")
}

pub fn grasp_repos() -> PathBuf {
    base_dir_path().join("repos")
}
