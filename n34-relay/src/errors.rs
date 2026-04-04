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

use nostr_database::DatabaseError;

use crate::relay::GrpcError;
use crate::relay::RhaiPluginsError;

/// Relay `Result` type
pub type RelayResult<T> = Result<T, RelayError>;

/// Relay errors
#[derive(Debug, thiserror::Error)]
pub enum RelayError {
    #[error("IO Error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Database error: {0}")]
    Database(#[from] DatabaseError),
    #[error("gRPC initialization error: {0}")]
    Grpc(#[from] GrpcError),
    #[error("Rhai error: {0}")]
    Rhai(#[from] RhaiPluginsError),
    #[error("File system Error: path `{0}` {1}")]
    Fs(PathBuf, String),
    #[error("Config error: {0}")]
    Config(String),
}
