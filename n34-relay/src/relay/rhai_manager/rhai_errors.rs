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

use std::io;
use std::path::PathBuf;

use rhai::EvalAltResult;

/// A result type
pub type RhaiPluginsResult<T> = Result<T, RhaiPluginsError>;

#[derive(Debug, thiserror::Error)]
pub enum RhaiPluginsError {
    #[error("Failed to read script `{1}`: {0}")]
    ReadScript(io::Error, PathBuf),
    #[error("Failed to compile script `{1}`: {0}")]
    CompileScript(Box<EvalAltResult>, PathBuf),
    #[error("Missing `IS_ALL` constant in plugin `{0}`")]
    MissingIsAll(String),
    #[error("`IS_ALL` constant in plugin `{0}` has type `{1}`, expected boolean")]
    InvalidIsAllType(String, &'static str),
    #[error("Plugin `{0}` is missing `admit_event` and `admit_query` functions")]
    MissingPluginFunction(String),
}
