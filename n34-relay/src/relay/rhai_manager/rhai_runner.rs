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

use std::{fmt, fs, sync::Arc};

use nostr::{event::Event, filter::Filter};
use rhai::{
    Dynamic,
    Engine,
    OptimizationLevel,
    Scope,
    module_resolvers::FileModuleResolver,
    serde::DynamicSerializer,
};
use serde::Serialize;

use super::{RhaiPlugin, RhaiPlugins, rhai_errors::RhaiPluginsResult};
use crate::{pathes, relay_config::RelayConfig};

/// Rhai plugins manager
pub struct RhaiPluginsRunner {
    plugins: RhaiPlugins,
    engine:  Engine,
}

impl RhaiPluginsRunner {
    /// Create the engine
    fn load_engine() -> Engine {
        let mut engine = Engine::new();
        engine.set_allow_loop_expressions(false);
        engine.set_strict_variables(true);
        engine.set_optimization_level(OptimizationLevel::Simple);
        engine.set_module_resolver(FileModuleResolver::new_with_path(pathes::rhai_plugins_dir()));
        engine
    }

    /// Create a new `[RhaiPluginsRunner]` instance
    pub fn new(config: &RelayConfig) -> RhaiPluginsResult<Self> {
        let engine = Self::load_engine();
        let rhai_plugins_dir = pathes::rhai_plugins_dir();
        let _ = fs::create_dir_all(&rhai_plugins_dir);

        let plugin_name_path = |name: &str| {
            let mut path = rhai_plugins_dir.join(name);
            path.set_extension("rhai");
            path
        };

        let plugins = config
            .plugins
            .rhai
            .plugins
            .iter()
            .map(|plugin_name| {
                Ok(Arc::new(RhaiPlugin::from_file(
                    &engine,
                    plugin_name,
                    plugin_name_path(plugin_name),
                )?))
            })
            .collect::<RhaiPluginsResult<Vec<_>>>()?;

        Ok(Self {
            engine,
            plugins: RhaiPlugins::new(plugins),
        })
    }

    fn admit_something<T>(
        &self,
        all_something: &[Arc<RhaiPlugin>],
        any_something: &[Arc<RhaiPlugin>],
        fn_name: &str,
        argument: &T,
    ) -> Result<(), String>
    where
        T: Serialize + fmt::Debug,
    {
        // all all-plugins must pass
        for plugin in all_something {
            execute_rhai_fn(&plugin.name, fn_name, &self.engine, &plugin.ast, argument)?;
        }

        // Pass if there is no any-plugins
        if self.plugins.any_write.is_empty() {
            return Ok(());
        }

        // to store the first rejection message
        let mut reject_msg = None;
        for plugin in any_something {
            match execute_rhai_fn(&plugin.name, fn_name, &self.engine, &plugin.ast, argument) {
                // One any-plugin must pass to return `Ok(())`
                Ok(()) => return Ok(()),
                // store the first rejection message
                Err(msg) if reject_msg.is_none() => reject_msg = Some(msg),
                _ => {}
            }
        }

        // No any-plugin passed, returns the first rejection message
        Err(reject_msg.unwrap())
    }

    /// Processes the given event by executing the `admit_event` function across
    /// all plugins. For success, all plugins in `all_write` must pass, and
    /// at least one plugin in `any_write` must pass. If all checks pass,
    /// `Ok(())` is returned. Otherwise, the first rejection message is returned
    /// as an error.
    pub fn admit_event(&self, event: &Event) -> Result<(), String> {
        self.admit_something(
            &self.plugins.all_write,
            &self.plugins.any_write,
            "admit_event",
            event,
        )
    }

    /// Processes the given query by executing the `admit_query` function across
    /// all plugins. For success, all plugins in `all_query` must pass, and
    /// at least one plugin in `any_query` must pass. If all checks pass,
    /// `Ok(())` is returned. Otherwise, the first rejection message is returned
    /// as an error.
    pub fn admit_query(&self, query: &Filter) -> Result<(), String> {
        self.admit_something(
            &self.plugins.all_query,
            &self.plugins.any_query,
            "admit_query",
            query,
        )
    }
}

/// Convert the type to rhai map. In case of error, log it and returns error
fn to_rhai_map<T: Serialize + fmt::Debug>(v: &T) -> Option<Dynamic> {
    match v.serialize(&mut DynamicSerializer::new(Dynamic::UNIT)) {
        Ok(v) => Some(v),
        Err(err) => {
            // This shouldn't happen
            tracing::error!(
                event = ?v,
                "Failed to serialize an event to rhai map, {err}"
            );
            None
        }
    }
}

/// Runs a Rhai function with the provided argument using the given engine and
/// AST.
///
/// Returns the Result<(), reject_msg>
fn execute_rhai_fn<T: Serialize + fmt::Debug>(
    plugin_name: &str,
    fn_name: &str,
    engine: &Engine,
    ast: &rhai::AST,
    argument: &T,
) -> Result<(), String> {
    let Some(rhai_map) = to_rhai_map(argument) else {
        return Ok(());
    };

    if let Err(err) = engine.call_fn::<()>(&mut Scope::new(), ast, fn_name, (rhai_map,)) {
        match *err {
            rhai::EvalAltResult::ErrorRuntime(dynamic, ..) => {
                match dynamic.into_string() {
                    Ok(s) => return Err(s),
                    Err(type_name) => {
                        tracing::error!(
                            plugin = %plugin_name,
                            fn = %fn_name,
                            "Rhai plugin threw a `{type_name}` instead of a string as a rejection message",
                        );
                    }
                }
            }
            _ => {
                tracing::error!(
                    plugin = %plugin_name,
                    fn = %fn_name,
                    "Rhai plugin encountered an error: {err}",
                );
            }
        }
    }
    Ok(())
}
