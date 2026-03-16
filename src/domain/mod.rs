// src/domain/mod.rs - Core Domain Layer
//
// This module contains the pure business logic, domain entities, and ports (traits).
// It has NO dependencies on infrastructure (databases, web frameworks, etc.)
//
// The models module provides unified types shared between server and client.

// Always available
pub mod bulk_operations;
pub mod errors;
pub mod graph_schema;
pub mod models;
pub mod plugin_api;
#[cfg(feature = "server")]
pub mod proxy_core;

// Server-only
#[cfg(feature = "server")]
pub mod ports;
#[cfg(feature = "server")]
pub mod services;

pub use errors::*;
pub use models::*;

#[cfg(feature = "server")]
pub use ports::*;
#[cfg(feature = "server")]
pub use services::*;
