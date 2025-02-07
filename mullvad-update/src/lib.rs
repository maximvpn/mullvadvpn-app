//! Support functions for securely installing or updating Mullvad VPN

pub mod api;
pub mod app;
pub mod fetch;
pub mod verify;

/// Parser and serializer for version metadata
pub mod format;
