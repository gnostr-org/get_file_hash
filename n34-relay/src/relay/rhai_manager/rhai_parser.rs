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

use rhai::AST;

use super::{RhaiPluginType, rhai_errors::RhaiPluginsResult};
use crate::relay::rhai_manager::rhai_errors::RhaiPluginsError;

/// Retrieves the `IS_ALL` boolean value from the AST. Searches for the `IS_ALL`
/// constant and returns its boolean value if found.
#[inline]
pub fn is_all(plugin_name: &str, ast: &AST) -> RhaiPluginsResult<bool> {
    ast.iter_literal_variables(true, false)
        .find_map(|(name, _, value)| {
            if name == "IS_ALL" {
                Some(value.as_bool().map_err(|type_name| {
                    RhaiPluginsError::InvalidIsAllType(plugin_name.to_owned(), type_name)
                }))
            } else {
                None
            }
        })
        .ok_or_else(|| RhaiPluginsError::MissingIsAll(plugin_name.to_owned()))?
}

/// Determines the type of the plugin based on the functions it contains. A
/// plugin can be `WRITE`, `QUERY`, or `BOTH` depending on the presence of
/// `admit_event` and `admit_query` functions.
#[inline]
pub fn plugin_type(plugin_name: &str, ast: &AST) -> RhaiPluginsResult<RhaiPluginType> {
    match (
        contains_function(ast, "admit_event", 1),
        contains_function(ast, "admit_query", 1),
    ) {
        (true, true) => Ok(RhaiPluginType::Both),
        (true, false) => Ok(RhaiPluginType::Write),
        (false, true) => Ok(RhaiPluginType::Query),
        (false, false) => {
            Err(RhaiPluginsError::MissingPluginFunction(
                plugin_name.to_owned(),
            ))
        }
    }
}

/// Checks if a function with the specified name and number of parameters exists
/// in the AST.
#[inline]
fn contains_function(ast: &AST, fn_name: &str, params_num: usize) -> bool {
    ast.iter_functions()
        .any(|function| function.name == fn_name && function.params.len() == params_num)
}
