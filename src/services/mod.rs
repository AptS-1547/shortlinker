//! Service layer for business logic
//!
//! This module provides unified business logic that can be shared between
//! different interfaces (HTTP API, IPC, CLI).

mod analytics_service;
pub mod geoip;
mod link_service;
mod user_agent_store;

pub use analytics_service::*;
pub use geoip::{GeoInfo, GeoIpLookup, GeoIpProvider};
pub use link_service::*;
pub use user_agent_store::{get_user_agent_store, set_global_user_agent_store, UserAgentStore};
