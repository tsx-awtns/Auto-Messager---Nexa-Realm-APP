# Getting started

## 1. Requirements

- Rust (the toolchain used by Nexus Realm 6.2.0 is `stable`).
- The official WebAssembly target:

```
rustup target add wasm32-wasip1
```

`nexus-plugin build` checks that the target's standard library is installed and
refuses to run otherwise. It never installs toolchains or targets for you.

## 2. Build the CLI

From the repository root:

```
cargo build --manifest-path plugin-sdk/Cargo.toml -p nexus-plugin-cli
```

The binary is `plugin-sdk/target/debug/nexus-plugin.exe` (or `nexus-plugin` on
other platforms). Add it to `PATH`, or call it by path.

## 3. Create a project

```
nexus-plugin new hello-nexus
cd hello-nexus
```

The generator creates `Cargo.toml`, `plugin.json`, `src/lib.rs`, `README.md`,
`.gitignore`, and an empty `assets/` directory. The plugin id defaults to
`com.example.<project-name>`; pass `--id <reverse-domain-id>` when that is not
valid or not what you want.

Inside this repository the generated project depends on the SDK by relative
path, so no crates need to be published first. Outside the repository the
generator writes a version dependency instead.

## 4. Implement the plugin

```rust
use nexus_plugin_sdk::prelude::*;

#[derive(Default)]
pub struct Plugin;

impl NexusPlugin for Plugin {
    fn metadata(&self) -> PluginMetadata {
        PluginMetadata::new("com.example.hello-nexus", "Hello Nexus", "1.0.0")
            .permission(Permission::EventsSubscribe)
            .subscribe(EventKind::AppReady)
    }

    fn on_load(&mut self, _context: &PluginContext) -> PluginResult<()> {
        Ok(())
    }

    fn on_event(&mut self, _context: &PluginContext, event: PluginEvent) -> PluginResult<()> {
        match event.kind() {
            EventKind::AppReady => Ok(()),
            _ => Ok(()),
        }
    }
}

nexus_plugin_sdk::export_plugin!(Plugin);
```

`PluginMetadata` must describe the same plugin id and version as `plugin.json`.
Subscribing to any event requires the `events.subscribe` permission.

## 5. Build, validate, package

```
nexus-plugin build      # cargo build --release --target wasm32-wasip1, stages plugin.wasm
nexus-plugin validate   # structural validation of the project
nexus-plugin pack       # writes dist/<name>-<version>.nexusplugin
```

Useful options:

- `build --offline` builds without network access.
- `build --target <triple>` overrides the target (the official target stays
  `wasm32-wasip1`).
- `validate --package <file.nexusplugin>` validates an existing package,
  including its integrity metadata.
- `pack --out <dir>` writes the package somewhere else.

## 6. What happens next

The package is unsigned and unverified. Nexus Realm 6.2.0 does not install
`.nexusplugin` files yet — installation, publisher trust, and consent are later
phases — but it can already admit and execute a plugin that sits in the
`UserPlugins` layout it trusts, inside a capability-free sandbox with bounded
memory, fuel, and time. `nexus-plugin validate` reports structural findings only
and never claims that a plugin is trusted.

When a plugin runs it can deliver log lines through the Host API:

```rust
fn on_load(&mut self, context: &PluginContext) -> PluginResult<()> {
    context.log().info("plugin loaded")
}
```

Logging requires the `logs.write` permission, and the message must be UTF-8,
non-empty, control-character free, and at most 1024 bytes. A denied call returns
`PluginError::PermissionDenied` instead of reaching Nexus Realm.
