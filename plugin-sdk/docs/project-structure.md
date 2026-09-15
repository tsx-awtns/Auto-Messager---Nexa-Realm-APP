# Project structure

## A plugin project

```
hello-nexus/
  Cargo.toml     cdylib crate, depends on nexus-plugin-sdk only
  plugin.json    Plugin Manifest V1 (validated by Nexus Realm)
  src/lib.rs     the plugin implementation
  README.md      project documentation
  .gitignore     ignores target/, dist/, and the staged plugin.wasm
  assets/        optional static assets, packaged as-is
  plugin.wasm    produced by `nexus-plugin build` (not committed)
  dist/          produced by `nexus-plugin pack` (not committed)
```

Rules the tooling enforces:

- the crate is a `cdylib` built for `wasm32-wasip1`;
- the only dependency a plugin needs is `nexus-plugin-sdk`;
- no Tauri, no Nexus Realm internals, no native libraries, no build scripts that
  emit executables;
- only `plugin.json`, `plugin.wasm`, and `assets/**` can be packaged;
- every path component in the package must be ASCII, must not use `..`, must not
  be absolute, must not contain `\`, and must not be a Windows device name.

## The SDK workspace

```
plugin-sdk/
  Cargo.toml                    workspace: resolver 2, three member crates
  crates/nexus-plugin-api/      the public contract (see below)
  crates/nexus-plugin-sdk/      developer-facing crate
  crates/nexus-plugin-cli/      the `nexus-plugin` binary and libraries
  templates/basic-plugin/       the official template used by `nexus-plugin new`
  examples/hello-nexus/         a generated example project
  docs/                         this documentation
```

### `nexus-plugin-api`

The single authoritative definition of the public contract. Nexus Realm Core
consumes this crate by path, so the application and the SDK cannot drift:

- Plugin API version and schema versions;
- `PluginManifest` (Manifest V1) and its validation rules;
- `PluginId` and `Sha256Hash`;
- the permission vocabulary and protected capability list;
- the public event vocabulary and `PublicEvent`;
- content limits, file names, and the forbidden native payload policy;
- plugin-relative path syntax;
- the Plugin API v1 ABI status codes and message types;
- the `.nexusplugin` package integrity model and its canonical JSON form.

### `nexus-plugin-sdk`

What plugin authors use: `NexusPlugin`, `PluginContext`, `PluginMetadata`,
`PluginEvent`, `PluginError`, `PluginResult`, the `prelude`, and the
`export_plugin!` macro that emits the Plugin API v1 exports.

### `nexus-plugin-cli`

`nexus-plugin new|build|validate|pack|help|--version`. The command surface is
defined in `cli.rs`; each command returns a structured report so behaviour is
testable without parsing console output.
