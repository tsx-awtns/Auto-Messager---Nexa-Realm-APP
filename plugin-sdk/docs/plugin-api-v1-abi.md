# Plugin API v1 ABI

This document defines the boundary between a plugin module and a Nexus Realm
host. It is implemented by `nexus-plugin-sdk` (module side) and defined by
`nexus-plugin-api::abi` (contract). Nexus Realm implements the host side in
`main-interface/src/plugins/runtime`, and executes only modules that pass
admission.

## Module shape

- WebAssembly **core** module (not a component), version 1.
- Target `wasm32-wasip1`, built as a `cdylib`.
- The module owns its linear memory and exports it as `memory`.
- Messages are UTF-8 JSON with a fixed schema and a size ceiling.

## Exports

| Export | Signature | Purpose |
| --- | --- | --- |
| `nexus_plugin_api_version` | `() -> u32` | `(major << 16) | (minor << 8) | patch`; `1.0.0` is `0x00010000` |
| `nexus_plugin_alloc` | `(u32) -> u32` | allocate a host-writable buffer; `0` means refusal |
| `nexus_plugin_free` | `(u32, u32)` | release a buffer with the size it was allocated with |
| `nexus_plugin_init` | `(u32, u32) -> i32` | receive an init request and load the plugin |
| `nexus_plugin_describe` | `(u32, u32) -> i32` | write the plugin description into a host buffer; returns bytes written, or a negative status |
| `nexus_plugin_event` | `(u32, u32) -> i32` | receive one public event |
| `nexus_plugin_shutdown` | `() -> i32` | release the plugin instance (idempotent) |

All seven are emitted by `nexus_plugin_sdk::export_plugin!(Plugin)`. Nexus Realm
rejects a module that does not export all of them.

## Status codes

| Code | Name | Meaning |
| --- | --- | --- |
| 0 | `Ok` | success |
| -1 | `InvalidArgument` | null or zero-length buffer, size above the ceiling, or a plugin id mismatch |
| -2 | `InvalidState` | call is not valid in the current lifecycle state |
| -3 | `MalformedMessage` | message is not valid JSON, has unknown fields, or is missing fields |
| -4 | `LimitExceeded` | the message does not fit the provided capacity or a documented limit is exceeded |
| -5 | `UnsupportedVersion` | schema version or API major version is not supported |
| -6 | `PluginError` | the plugin reported a failure |
| -7 | `Internal` | the plugin could not proceed (for example a poisoned lock) |
| -8 | `PermissionDenied` | a host call was denied because no grant covers it |

## Memory ownership

1. The host allocates every buffer it wants a plugin to read or write, by calling
   the module's `nexus_plugin_alloc`.
2. The host writes the payload into that buffer and calls the export with the
   buffer address and length.
3. The host frees the buffer with `nexus_plugin_free` and the same size.

The host therefore never dereferences a plugin-provided pointer or length. Plugin
output is written only into a host-allocated buffer with a host-specified
capacity, and the returned length is validated against that capacity before the
host reads anything.

## Bounded messages

Messages are capped at 64 KiB (`MAX_ABI_MESSAGE_BYTES`). The SDK checks the
address, the length, and the capacity before touching memory, and refuses
`0`, `0`-length, and over-sized arguments with `InvalidArgument`. Every message
type uses `deny_unknown_fields`, so a malformed message is rejected rather than
partially interpreted, and a failed call never leaves a half-initialized
instance behind.

**Init request** (host to plugin)

```json
{ "schemaVersion": 1, "apiVersion": "1.0.0", "pluginId": "com.example.my-plugin" }
```

The plugin verifies the schema version, the API major version, and that the
requested plugin id matches the id in its own metadata.

**Describe response** (plugin to host)

```json
{
  "schemaVersion": 1,
  "id": "com.example.my-plugin",
  "name": "My Plugin",
  "version": "1.0.0",
  "apiVersion": "1.0.0",
  "permissions": ["events.subscribe"],
  "subscriptions": ["app.ready"]
}
```

**Event** (host to plugin)

```json
{ "schemaVersion": 1, "sequence": 42, "occurredAtMs": 1789301904707, "kind": "app.ready" }
```

## Lifecycle

```
nexus_plugin_api_version
        |
nexus_plugin_init        (fails => no instance exists)
        |
nexus_plugin_describe    (identity cross-check)
        |
nexus_plugin_event ...   (0..n)
        |
nexus_plugin_shutdown    (idempotent)
```

## Host calls

Host calls live in one explicit import namespace, `nexus_host`, and every call is
permission-checked by the runtime. Phase 3 defines exactly one call:

```
nexus_host_log(level: u32, pointer: u32, length: u32) -> i32
```

`level` is `1` (info), `2` (warn), or `3` (error). `pointer` and `length`
describe a UTF-8 message in the plugin's own linear memory, capped at 1024 bytes
(`MAX_HOST_LOG_BYTES`). The host validates the level, the length, and the
`logs.write` grant, reads the bytes with bounds checks, validates UTF-8, replaces
control characters, and appends the plugin id itself — a plugin cannot spoof its
identity, choose a destination path, or inject log structure. Failure returns
`PermissionDenied`, `InvalidArgument`, `MalformedMessage`, or `LimitExceeded`;
the plugin decides whether to continue or fail. The SDK wraps this as
`ctx.log().info("...")`, `ctx.log().warn("...")`, and `ctx.log().error("...")`.

A module that imports anything else from `nexus_host`, or any WASI function
outside the four listed below, is rejected during admission.

## WASI imports in a built module

A plugin built with `std` and `panic = "abort"` imports exactly four WASI
Preview 1 functions today:

```
wasi_snapshot_preview1::environ_get
wasi_snapshot_preview1::environ_sizes_get
wasi_snapshot_preview1::fd_write
wasi_snapshot_preview1::proc_exit
```

There is no filesystem, clock, random, socket, or process import. `nexus-plugin
build` and `validate` report the observed import set. Nexus Realm provides
exactly this capability-free surface — an empty environment, a discarded
bounded descriptor write, and exit mapped to a trap — and rejects any module
that imports more.

## Error reporting

Plugins report failures through the closed status codes above. There is no
free-form error string crossing the boundary, so plugin-authored text never
reaches host logs or user interfaces.
