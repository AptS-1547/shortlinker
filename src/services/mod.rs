//! Service layer for business logic
//!
//! This module provides unified business logic that can be shared between
//! different interfaces (HTTP API, IPC, CLI).

mod link_service;

pub use link_service::*;
