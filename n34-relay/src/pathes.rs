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

use std::{env, path::PathBuf};

/// Name of the environment variable that can override the base directory
const BASE_DIR_ENV_VAR: &str = "N34_RELAY_BASE_DIR";

/// Default base directory path when no environment variable is set
const DEFAULT_BASE_DIR: &str = "/etc/n34-relay";

/// Gets the base directory path, using either the environment variable
/// `N34_RELAY_BASE_DIR` if set, or falling back to the default `/etc/n34-relay`
pub fn base_dir_path() -> PathBuf {
    env::var(BASE_DIR_ENV_VAR)
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from(DEFAULT_BASE_DIR))
}

/// Path to the config file located at `<base_dir>/config.toml`.
pub fn config_file_path() -> PathBuf {
    base_dir_path().join("config.toml")
}

/// Path to the LMDB directory within the base directory.
pub fn lmdb_dir_path() -> PathBuf {
    base_dir_path().join("lmdb")
}

/// Path to the logs file
pub fn logs_file_path() -> PathBuf {
    base_dir_path().join("logs.log")
}

/// Path to the html homepage file
pub fn homepage_file_path() -> PathBuf {
    base_dir_path().join("homepage.html")
}

/// Path to rhai plugins directory
pub fn rhai_plugins_dir() -> PathBuf {
    base_dir_path().join("rhai-plugins")
}

/// Path to GRASP repositories
pub fn grasp_repos() -> PathBuf {
    base_dir_path().join("repos")
}
