# Nexus Realm Plugin SDK

Official developer kit for Nexus Realm plugins. Plugins are written in Rust,
compiled to WebAssembly for the Plugin API v1 target `wasm32-wasip1`, and
packaged as a `.nexusplugin` file.

> Nexus Realm 6.2.0 admits and executes an admitted plugin inside a
> capability-free WebAssembly sandbox with bounded memory, fuel, and time. This
> SDK produces structurally valid, unsigned packages: installation, publisher
> trust, and consent arrive in the next phase, so a structurally valid plugin is
> still not TRUSTED.

## Workflow

```
Build the CLI         cargo build --manifest-path plugin-sdk/Cargo.toml -p nexus-plugin-cli
Add the WASM target   rustup target add wasm32-wasip1
Create a project      nexus-plugin new my-plugin
Write Rust            src/lib.rs + plugin.json
Build                 nexus-plugin build
Validate              nexus-plugin validate
Package               nexus-plugin pack
```

The result is `dist/<name>-<version>.nexusplugin`, a deterministic ZIP container
holding `plugin.json`, `plugin.wasm`, optional `assets/`, and
`package-integrity.json`.

At runtime a plugin can subscribe to public events, receive them through its
`on_event` callback, and write bounded log lines with `ctx.log().info(...)`
(which requires the `logs.write` command). No other host capability exists yet.

## Workspace layout

```
plugin-sdk/
  crates/nexus-plugin-api/   authoritative public contract (shared with Nexus Realm)
  crates/nexus-plugin-sdk/   developer-facing crate: trait, metadata, ABI shims
  crates/nexus-plugin-cli/   the `nexus-plugin` binary
  templates/basic-plugin/    the project template used by `nexus-plugin new`
  examples/hello-nexus/      one minimal example (build/pack validation)
  docs/                      developer documentation
```

## Documentation

- [Getting started](docs/getting-started.md)
- [Project structure](docs/project-structure.md)
- [Manifest V1](docs/manifest-v1.md)
- [Permissions V1](docs/permissions-v1.md)
- [Events V1](docs/events-v1.md)
- [Build and packaging](docs/build-and-packaging.md)
- [`.nexusplugin` format](docs/nexusplugin-format.md)
- [Plugin API v1 ABI](docs/plugin-api-v1-abi.md)
- [Security model](docs/security-model.md)
- [API versioning](docs/api-versioning.md)

## Checks

```
cargo check --manifest-path plugin-sdk/Cargo.toml --offline
cargo test  --manifest-path plugin-sdk/Cargo.toml --offline
```

The Nexus Realm application build does not depend on this workspace; it consumes
only `crates/nexus-plugin-api` as a path dependency.

## License

The Plugin Development Kit — `nexus-plugin-api`, `nexus-plugin-sdk`,
`nexus-plugin-package`, `nexus-plugin-cli`, the project template, the example
plugin and this documentation — is licensed under the MIT license. See
[LICENSE](LICENSE).

- You keep ownership of the plugins you write. Using this kit does not transfer
  ownership of your plugin code to Nexus Realm.
- MIT grants copyright permissions only. It does not grant rights in the Nexus
  Realm application, its source, its assets or its branding, and it grants no
  official status, publisher verification or endorsement.
- The Nexus Realm application is licensed separately from this kit. Publishing
  this kit does not change the application's license or make the application
  open source.
- Plugins you publish must comply with the law that applies to you and with the
  licenses of the components and services your plugin uses.
