//! Service layer for business logic
//!
//! This module provides unified business logic that can be shared between
//! different interfaces (HTTP API, IPC, CLI).

mod analytics_service;
pub mod geoip;
mod link_service;

pub use analytics_service::*;
pub use geoip::{GeoInfo, GeoIpLookup, GeoIpProvider};
pub use link_service::*;
