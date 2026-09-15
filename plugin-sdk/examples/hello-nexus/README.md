# Nexus Realm plugin project

This project is a Nexus Realm plugin written in Rust. It compiles to WebAssembly
for the official Plugin API v1 target `wasm32-wasip1`.

## Layout

```
Cargo.toml     Rust crate manifest (cdylib, no Tauri, no Nexus Realm internals)
plugin.json    Plugin Manifest V1 validated by Nexus Realm
src/lib.rs     Plugin implementation
assets/        Optional static assets packaged with the plugin
```

## Workflow

```
nexus-plugin build      # cargo build --release --target wasm32-wasip1, stages plugin.wasm
nexus-plugin validate   # structural validation of the project
nexus-plugin pack       # writes dist/<name>-<version>.nexusplugin
```

The build requires the WebAssembly target to be installed first:

```
rustup target add wasm32-wasip1
```

## What this plugin can and cannot do

Plugins may extend Nexus Realm. Plugins may never modify Nexus Realm: there is no
API for Core files, Nexus Integrity, root-of-trust material, trusted keys,
updater trust, raw credentials, DPAPI, process or shell execution, native
libraries, or global filesystem access. The permissions declared in
`plugin.json` are requestable identifiers only; they are granted by Nexus Realm,
never by the plugin.

## Status

Nexus Realm 6.2.0 can now admit and execute an admitted plugin inside a
capability-free WebAssembly sandbox with bounded memory, fuel, and time. This
template uses the one Host API available in this phase: bounded logging through
`ctx.log()`, which requires the `logs.write` permission. Publisher trust,
installation, and the broader Host API arrive later; a structurally valid
plugin is still not TRUSTED.

## Development notes

`PluginMetadata` must match the identifier and version in `plugin.json`. Keeping
the metadata, the manifest, and the packaged content consistent is what
`nexus-plugin validate` checks.
