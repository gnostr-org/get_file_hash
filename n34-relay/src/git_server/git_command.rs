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

use std::{path::Path, process::Stdio};

use axum::body::Body;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    process::{Child, ChildStderr, Command},
};
use tokio_util::io::ReaderStream;

use crate::git_server::ServiceName;

/// A helper for executing git commands with specific paths.
pub struct GitCommand<'a> {
    git_path: &'a str,
    git_repo: &'a Path,
}

impl<'a> GitCommand<'a> {
    /// Creates a new [`GitCommand`] instance with the given git path and
    /// repository.
    pub fn new(git_path: &'a str, git_repo: &'a Path) -> Self {
        Self { git_path, git_repo }
    }

    /// Spawns a git process with the provided arguments.
    fn spawn_git(&self, args: &[&str], v2: bool) -> Option<Child> {
        let mut command = Command::new(self.git_path);
        // from GRASP protocol:
        // MUST include `allow-reachable-sha1-in-want` and
        // `allow-tip-sha1-in-want` in advertisement and serve available oids.
        command.args(["-c", "uploadpack.allowTipSHA1InWant=true"]);
        command.args(["-c", "uploadpack.allowReachableSHA1InWant=true"]);

        command.args(args);
        command.stdin(Stdio::piped());
        command.stdout(Stdio::piped());
        command.stderr(Stdio::piped());
        command.current_dir(self.git_repo);

        if v2 {
            command.env("GIT_PROTOCOL", "version=2");
        }

        command.spawn().ok()
    }

    /// Sends a request body to a git command and returns its output as a
    /// stream.
    async fn pass_request_to_git(
        &self,
        args: &[&'static str],
        body: &[u8],
        v2: bool,
    ) -> Result<Body, &'static str> {
        let Some(mut process) = self.spawn_git(args, v2) else {
            return Err("Failed to run git command");
        };

        let mut stdin = process.stdin.take().expect("git stdin");
        if let Err(err) = stdin.write_all(body).await {
            tracing::error!(args = ?args, error = %err, "Failed to write the request body to git stdin");
            return Err("Failed to pass the request body to git");
        }
        drop(stdin);

        let stderr = process.stderr.take().expect("git stderr");
        let stdout = process.stdout.take().expect("git stdout");
        wait_process(process, stderr);
        Ok(Body::from_stream(ReaderStream::new(stdout)))
    }

    /// Executes a git command with the provided arguments and returns its
    /// output. If a body is needed, use [Self::pass_request_to_git]
    /// instead.
    async fn call_git(&self, args: &[&'static str], v2: bool) -> Result<Vec<u8>, &'static str> {
        let Some(process) = self.spawn_git(args, v2) else {
            return Err("Failed to run git command");
        };

        process
            .wait_with_output()
            .await
            .map(|o| o.stdout)
            .map_err(|err| {
                tracing::error!(args = ?args, error = %err, "Failed to get `git` output");
                "Failed to run `git` command"
            })
    }

    /// Executes the `receive-pack` git command with the provided body.
    pub async fn receive_pack(&self, body: &[u8]) -> Result<Body, &'static str> {
        self.pass_request_to_git(&["receive-pack", "--stateless-rpc", "."], body, false)
            .await
    }

    /// Executes the `upload-pack` git command with the provided body.
    pub async fn upload_pack(&self, body: &[u8], v2: bool) -> Result<Body, &'static str> {
        self.pass_request_to_git(&["upload-pack", "--stateless-rpc", "."], body, v2)
            .await
    }

    /// Executes the given service git command with `--advertise-refs` argument.
    pub async fn refs(&self, service: &ServiceName, v2: bool) -> Result<Body, &'static str> {
        let body_bytes = self
            .call_git(
                &[service.name(), "--stateless-rpc", "--advertise-refs", "."],
                v2,
            )
            .await?;

        Ok(Body::from(
            [service.pkt_line_header(), body_bytes.as_ref()].concat(),
        ))
    }
}

/// Spawns a task to wait for the process to complete and logs any errors.
///
/// If the process exits with non-zero status, reads and logs the stderr output.
fn wait_process(mut process: Child, mut stderr: ChildStderr) {
    tokio::spawn(async move {
        let pid = process.id().unwrap_or_default();
        match process.wait().await {
            Ok(status) => {
                if !status.success() {
                    let mut err = String::new();
                    _ = stderr.read_to_string(&mut err).await;
                    tracing::warn!(
                        "Git process (PID {pid}) exited with non-zero status: {} ({status})",
                        err.trim(),
                    );
                }
            }
            Err(e) => {
                tracing::error!("Error waiting for Git process (PID {pid}): {e}");
            }
        }
    });
}
