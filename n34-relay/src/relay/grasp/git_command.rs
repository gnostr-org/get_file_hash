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

use std::{io, path::Path, process::Output, time::Duration};

use tokio::{fs, process::Command};

/// Type alias for `Result<T, GraspGitError>`
type GitResult<T> = Result<T, GraspGitError>;

/// Grasp git command errors
#[derive(Debug, thiserror::Error)]
pub enum GraspGitError {
    #[error("GRASP git IO error: {0}")]
    Io(#[from] io::Error),
    #[error("GRASP bare repo error: {0}")]
    BareRepo(String),
    #[error("GRASP git command timeout: `{0:?}`")]
    GitTimeout(Vec<String>),
    #[error("GRASP error, repo doesn't exists: {0}")]
    RepoNotFound(String),
}

/// Git command implementation for GRASP operations
pub struct GraspGitCommand<'a> {
    /// Path to the git executable
    pub git_path:  &'a str,
    /// Path to the git repository where commands will be executed
    pub repo_path: &'a Path,
}

impl<'a> GraspGitCommand<'a> {
    /// Creates a new Git command.
    pub fn new(git_path: &'a str, repo_path: &'a Path) -> Self {
        Self {
            git_path,
            repo_path,
        }
    }

    /// Executes the Git command with the provided arguments. Times out after 5
    /// seconds.
    async fn run_git(&self, args: &[&str]) -> GitResult<Output> {
        if !self.repo_path.exists() {
            tracing::trace!("Creatring repo directory: {}", self.repo_path.display());
            fs::create_dir_all(self.repo_path).await?;
        }

        tracing::trace!("Run GRASP git command: {args:?}");
        tokio::time::timeout(
            Duration::from_secs(5),
            Command::new(self.git_path)
                .kill_on_drop(true)
                .current_dir(self.repo_path)
                .args(args)
                .output(),
        )
        .await
        .map_err(|_| GraspGitError::GitTimeout(args.iter().map(ToString::to_string).collect()))?
        .map_err(GraspGitError::from)
    }

    /// Initializes a new bare Git repository.
    pub async fn new_bare(&self, description: &str) -> GitResult<()> {
        tracing::trace!("Creatring bare git repo `{}`", self.repo_path.display());
        let output = self.run_git(&["init", "--bare", "--quiet", "."]).await?;

        if !output.status.success() {
            return Err(GraspGitError::BareRepo(
                String::from_utf8_lossy(&output.stderr).into_owned(),
            ));
        }

        self.update_description(description).await
    }

    /// Updates the repository's description by writing the given string to the
    /// 'description' file. Returns an error if the repository path doesn't
    /// exist or if the write operation fails.
    pub async fn update_description(&self, description: &str) -> GitResult<()> {
        if !self.repo_path.exists() {
            return Err(GraspGitError::RepoNotFound(
                self.repo_path.display().to_string(),
            ));
        }

        tracing::trace!("updating repo description: {}", self.repo_path.display());
        fs::write(self.repo_path.join("description"), description)
            .await
            .map_err(GraspGitError::from)
    }
}
