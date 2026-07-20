//! Mode routing
//!
//! This module provides the HTTP server entry point.
//!
//! CLI mode is now handled directly by clap in main.rs.

#[cfg(feature = "server")]
pub mod server;

// Re-export mode functions for convenience
#[cfg(feature = "server")]
pub use server::run_server;
