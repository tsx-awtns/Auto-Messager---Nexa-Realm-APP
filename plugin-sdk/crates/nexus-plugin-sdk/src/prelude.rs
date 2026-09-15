// Nexus Realm
// File: prelude.rs
// Purpose: Single import surface for plugin authors

pub use crate::export_plugin;
pub use crate::plugin::{
    NexusPlugin, PluginContext, PluginError, PluginEvent, PluginLogger, PluginMetadata,
    PluginResult,
};
pub use nexus_plugin_api::{ApiVersion, EventKind, Permission, PluginId, PLUGIN_API_VERSION};
