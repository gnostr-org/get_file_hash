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

use std::{fs, path::Path, sync::Arc};

use rhai::{Engine, Scope};

pub use self::rhai_caller::*;
pub use self::rhai_errors::*;
pub use self::rhai_runner::*;
pub use self::rhai_worker::*;
use crate::relay_config::RelayConfig;

/// Rhai function caller
mod rhai_caller;
/// Rhai plugin manager errors
mod rhai_errors;
/// Parse a rhai script
mod rhai_parser;
/// The runner that run rhai scripts
mod rhai_runner;
/// The worker, who control the runner
mod rhai_worker;

/// Plugin type for Rhai scripts, indicating if they handle writes, queries, or
/// both.
pub enum RhaiPluginType {
    Write,
    Query,
    Both,
}

/// A Rhai script plugin.
///
/// The priority is either ALL (is_all=true) or ANY (is_all=false).
/// Plugins can handle events, queries, or both depending on their functions.
pub struct RhaiPlugin {
    /// Plugin name without the '.rhai' extension
    name:        String,
    /// Whether this plugin has ALL priority (true) or ANY priority (false)
    is_all:      bool,
    /// Type of operations this plugin handles
    plugin_type: RhaiPluginType,
    /// Parsed abstract syntax tree of the plugin
    ast:         rhai::AST,
}

/// A collection of Rhai plugins, organized by type and priority.
pub struct RhaiPlugins {
    /// Write-type plugins with `all` priority.
    all_write: Vec<Arc<RhaiPlugin>>,
    /// Write-type plugins with `any` priority.
    any_write: Vec<Arc<RhaiPlugin>>,
    /// Query-type plugins with `all` priority.
    all_query: Vec<Arc<RhaiPlugin>>,
    /// Query-type plugins with `any` priority.
    any_query: Vec<Arc<RhaiPlugin>>,
}

impl RhaiPluginType {
    /// Returns true if the type is either `Write` or `Both`.
    pub fn is_write(&self) -> bool {
        matches!(self, Self::Write | Self::Both)
    }

    /// Returns true if the type is either `Query` or `Both`.
    pub fn is_query(&self) -> bool {
        matches!(self, Self::Query | Self::Both)
    }
}

impl RhaiPlugin {
    /// Parse a rhai script from path
    pub fn from_file(
        engine: &Engine,
        plugin_name: &str,
        path: impl AsRef<Path>,
    ) -> RhaiPluginsResult<Self> {
        let path = path.as_ref();

        let ast = engine
            .compile_into_self_contained(
                &Scope::new(),
                fs::read_to_string(path)
                    .map_err(|err| RhaiPluginsError::ReadScript(err, path.to_path_buf()))?,
            )
            .map_err(|err| RhaiPluginsError::CompileScript(err, path.to_path_buf()))?;

        Ok(Self {
            name: plugin_name.to_owned(),
            is_all: rhai_parser::is_all(plugin_name, &ast)?,
            plugin_type: rhai_parser::plugin_type(plugin_name, &ast)?,
            ast,
        })
    }
}

impl RhaiPlugins {
    /// Create a new `[RhaiPlugins]` instance
    pub fn new(plugins: Vec<Arc<RhaiPlugin>>) -> Self {
        // Do the `filter_map` and collect
        let do_fm = |cond: fn(&Arc<RhaiPlugin>) -> bool| {
            plugins
                .iter()
                .filter_map(|p| if cond(p) { Some(Arc::clone(p)) } else { None })
                .collect()
        };

        Self {
            all_write: do_fm(|p| p.plugin_type.is_write() && p.is_all),
            any_write: do_fm(|p| p.plugin_type.is_write() && !p.is_all),
            all_query: do_fm(|p| p.plugin_type.is_query() && p.is_all),
            any_query: do_fm(|p| p.plugin_type.is_query() && !p.is_all),
        }
    }
}

/// Gets the manager if Rhai plugins are configured, otherwise returns None.
/// Logs an error if initialization fails.
pub async fn maybe_manager(config: &RelayConfig) -> Option<PluginCaller> {
    if config.plugins.rhai.plugins.is_empty() {
        tracing::info!("There is no rhai plugins, skipping initialization");
        return None;
    }

    let runner = match RhaiPluginsRunner::new(config) {
        Ok(runner) => Arc::new(runner),
        Err(err) => {
            tracing::error!("Failed to initialize the Rhai plugins runner: {err}");
            return None;
        }
    };

    let (worker_tx, worker_rx) = flume::bounded(1024);
    let caller = PluginCaller(worker_tx);

    for _ in 0..config.plugins.rhai.workers.get() {
        RhaiWorker::create(worker_rx.clone(), Arc::clone(&runner));
    }

    Some(caller)
}
