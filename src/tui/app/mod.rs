//! App module: state and business logic
//!
//! This module contains all app state and operations, organized by functionality:
//! - state: Core App struct and basic state management
//! - navigation: Selection and navigation logic
//! - validation: Input validation
//! - link_operations: CRUD operations for links
//! - file_operations: Import/export and file browser

mod file_operations;
mod link_operations;
mod navigation;
mod state;
mod validation;

// Re-export main types
pub use state::{App, CurrentScreen, CurrentlyEditing};
