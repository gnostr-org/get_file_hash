//! Plugins Service Example  
//!
//! A basic guide to designing your plugins service. Feel free to refine or
//! improve this approach.

use std::{
    collections::HashMap,
    net::{Ipv4Addr, SocketAddr, SocketAddrV4},
    str::FromStr,
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

use axum::Extension;
use parking_lot::{Mutex, RwLock};
use strum::{AsRefStr, EnumIter, EnumString, IntoEnumIterator};
use tonic::{Code, Request, Response, Status, transport::Server};

use self::plugins_api::{
    Empty,
    PluginInfo,
    PluginPriority,
    PluginRequest,
    PluginResponse,
    PluginType,
    ServicePlugins,
    plugin_request::PluginRequestBody,
    plugin_response::PluginResponseBody,
    plugins_service_server::{PluginsService, PluginsServiceServer},
};

mod plugins_api {
    tonic::include_proto!("plugins");
}

struct Service;
struct PluginsState {
    max_tags_count: usize,
    events_per_day: usize,
    last_db_wibed:  RwLock<u64>,
    db:             Mutex<HashMap<String, usize>>,
}

impl PluginsState {
    /// Initialize the state
    fn new() -> Self {
        Self {
            max_tags_count: 30,
            events_per_day: 5,
            last_db_wibed:  RwLock::new(current_time()),
            db:             Mutex::new(HashMap::new()),
        }
    }
}

/// Service plugins.
#[derive(Debug, EnumIter, EnumString, AsRefStr)]
#[strum(serialize_all = "snake_case")]
enum Plugins {
    /// Restricts the number of tags allowed per event.
    TagsLimit,
    /// Sets a daily limit on the number of events that can be broadcasted for
    /// each public key.
    EventsPerDay,
    /// Prevents the listing of relay kind 1 events (text notes).
    NoKindOne,
}

impl Plugins {
    /// Returns the plugins info
    fn plugins_info() -> Vec<PluginInfo> {
        let mut plugins = Vec::new();
        for plugin in Self::iter() {
            plugins.push(PluginInfo {
                name:        plugin.as_ref().to_owned(),
                plugin_type: plugin.plugin_type() as i32,
                priority:    plugin.plugin_priority() as i32,
            });
        }
        plugins
    }

    /// Returns the plugin type
    #[inline]
    fn plugin_type(&self) -> PluginType {
        match self {
            Self::TagsLimit => PluginType::Write,
            Self::EventsPerDay => PluginType::Write,
            Self::NoKindOne => PluginType::Query,
        }
    }

    /// Returns the plugin priority
    #[inline]
    fn plugin_priority(&self) -> PluginPriority {
        PluginPriority::All
    }

    /// Run write plugin
    async fn run_write(
        &self,
        event: plugins_api::Event,
        state: Arc<PluginsState>,
    ) -> Result<(), String> {
        assert_eq!(self.plugin_type(), PluginType::Write);

        match self {
            Self::TagsLimit => run_tags_limit(event, state).await,
            Self::EventsPerDay => run_events_per_day(event, state).await,
            _ => unreachable!(), // You must make sure to include all write plugins
        }
    }

    /// Run query plugin
    async fn run_query(
        &self,
        filter: plugins_api::Filter,
        _state: Arc<PluginsState>,
    ) -> Result<(), String> {
        assert_eq!(self.plugin_type(), PluginType::Query);

        match self {
            Self::NoKindOne => run_no_kind_one(filter).await,
            _ => unreachable!(), // You must make sure to include all query plugins
        }
    }
}

#[tonic::async_trait]
impl PluginsService for Service {
    async fn run_plugin(
        &self,
        request: Request<PluginRequest>,
    ) -> Result<Response<PluginResponse>, Status> {
        let state = Arc::clone(request.extensions().get::<Arc<PluginsState>>().unwrap());
        let request = request.into_inner();

        let Ok(plugin) = Plugins::from_str(&request.plugin_name) else {
            return Err(Status::new(
                Code::InvalidArgument,
                format!("Unknown plugin: {}", request.plugin_name),
            ));
        };

        let Some(body) = request.plugin_request_body else {
            return Err(Status::new(
                Code::InvalidArgument,
                "Missing the request body",
            ));
        };

        let result = match body {
            PluginRequestBody::Event(event) => {
                if matches!(plugin.plugin_type(), PluginType::Query) {
                    return Err(Status::new(
                        Code::InvalidArgument,
                        format!(
                            "plugin '{}' is query plugin but received an event",
                            plugin.as_ref()
                        ),
                    ));
                }
                plugin.run_write(event, state).await
            }
            PluginRequestBody::Filter(filter) => {
                if matches!(plugin.plugin_type(), PluginType::Write) {
                    return Err(Status::new(
                        Code::InvalidArgument,
                        format!(
                            "plugin '{}' is write plugin but received a filter",
                            plugin.as_ref()
                        ),
                    ));
                }
                plugin.run_query(filter, state).await
            }
        };

        match result {
            Ok(()) => {
                Ok(Response::new(PluginResponse {
                    plugin_response_body: Some(PluginResponseBody::Accept(Empty {})),
                }))
            }
            Err(reject_msg) => {
                Ok(Response::new(PluginResponse {
                    plugin_response_body: Some(PluginResponseBody::RejectMsg(reject_msg)),
                }))
            }
        }
    }

    async fn get_plugins(&self, _: Request<Empty>) -> Result<Response<ServicePlugins>, Status> {
        Ok(Response::new(ServicePlugins {
            plugins: Plugins::plugins_info(),
        }))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let address = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 8080));
    println!("Service Address: 127.0.0.1:8080");

    Server::builder()
        .layer(Extension(Arc::new(PluginsState::new())))
        .add_service(PluginsServiceServer::new(Service))
        .serve(address)
        .await?;
    Ok(())
}

fn current_time() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(u64::MAX)
}

// --- Plugins --- //

async fn run_tags_limit(event: plugins_api::Event, state: Arc<PluginsState>) -> Result<(), String> {
    if event.tags.len() > state.max_tags_count {
        return Err(format!(
            "Too many tags in the event, the limit is {}",
            state.max_tags_count
        ));
    }

    Ok(())
}

async fn run_events_per_day(
    event: plugins_api::Event,
    state: Arc<PluginsState>,
) -> Result<(), String> {
    let mut db = state.db.lock();

    let count = db.entry(event.public_key).or_insert(1);

    if *count >= state.events_per_day {
        return Err("You exceeded your daily limit".to_owned());
    }

    *count += 1;

    // For testing purposes, the DB is cleared every 5 minutes. In a production
    // environment, this operation should occur every 24 hours, and a persistent
    // database should be used instead of an in-memory one. :)
    let now_time = current_time();
    if { now_time - *state.last_db_wibed.read() } >= 60 * 5 {
        *state.last_db_wibed.write() = now_time;
        *db = HashMap::new();
    }

    Ok(())
}

async fn run_no_kind_one(filter: plugins_api::Filter) -> Result<(), String> {
    if filter.ids.is_empty() && filter.authors.is_empty() && filter.kinds.contains(&1) {
        return Err("Exploring relay notes is not permitted".to_owned());
    }
    Ok(())
}
