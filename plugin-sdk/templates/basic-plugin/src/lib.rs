// Nexus Realm
// File: lib.rs
// Purpose: Basic Nexus Realm plugin

use nexus_plugin_sdk::prelude::*;

#[derive(Default)]
pub struct Plugin;

impl NexusPlugin for Plugin {
    fn metadata(&self) -> PluginMetadata {
        PluginMetadata::new("com.example.basic-plugin", "Basic Plugin", "1.0.0")
            .author("Example Developer")
            .description("Basic Nexus Realm plugin template")
            .permission(Permission::EventsSubscribe)
            .permission(Permission::LogsWrite)
            .subscribe(EventKind::AppReady)
    }

    fn on_load(&mut self, context: &PluginContext) -> PluginResult<()> {
        context.log().info("plugin loaded")
    }

    fn on_event(&mut self, context: &PluginContext, event: PluginEvent) -> PluginResult<()> {
        match event.kind() {
            EventKind::AppReady => context.log().info("application ready"),
            _ => Ok(()),
        }
    }
}

nexus_plugin_sdk::export_plugin!(Plugin);
