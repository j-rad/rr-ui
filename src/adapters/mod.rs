// src/adapters/mod.rs - Infrastructure Adapters
//
// This module contains all infrastructure-specific implementations
// (database adapters, web adapters, external service adapters)

pub mod atomic_config;
pub mod geo;
#[cfg(target_os = "linux")]
pub mod nftables_manager;
pub mod orchestrators;
pub mod repositories;
pub mod rustray_core;
pub mod singbox_core;
pub mod system_openwrt;
pub mod uds_manager;
pub mod validators;

pub use orchestrators::*;
pub use repositories::*;
pub use validators::*;
