# Plugin Manifest V1

`plugin.json` is the manifest Nexus Realm validates. It is defined once, in
`nexus-plugin-api::PluginManifest`, and both Nexus Realm Core and this SDK use
that definition.

```json
{
  "schemaVersion": 1,
  "id": "com.example.my-plugin",
  "name": "My Plugin",
  "version": "1.0.0",
  "author": "Example Developer",
  "description": "Example Nexus Realm plugin",
  "apiVersion": "1.0.0",
  "entry": "plugin.wasm",
  "permissions": ["events.subscribe"],
  "subscriptions": ["app.ready"]
}
```

## Rules

| Field | Rule |
| --- | --- |
| manifest size | at most 16 KiB before deserialization |
| unknown fields | rejected (strict schema) |
| duplicate fields | rejected |
| `schemaVersion` | exactly `1` |
| `id` | 5–128 ASCII bytes, at least three labels, each 1–63 bytes, lowercase, starting with a letter, ending alphanumeric, inner `-` allowed, no reserved device names |
| `name` | 1–80 bytes, no control characters, no surrounding whitespace |
| `author` | 1–120 bytes, same text rules |
| `description` | 1–1024 bytes, same text rules |
| `version` | strict SemVer, at most 64 bytes (prerelease and build metadata allowed) |
| `apiVersion` | strict SemVer and exactly the current Plugin API version |
| `entry` | exactly `plugin.wasm`, validated against the plugin path policy first |
| `permissions` | at most 12 entries, unique, from the Permission Model V1 vocabulary |
| `subscriptions` | at most 33 entries, unique, from the Public Event Contract V1 |

If `subscriptions` is not empty, `permissions` must contain `events.subscribe`.

## Generating a manifest

`PluginMetadata::to_manifest()` builds a manifest from the metadata a plugin
declares, and validates it before returning. `nexus-plugin new` writes the
generated project's `plugin.json` from the same type, so a generated manifest can
never contain a field the validator rejects.

## Compatibility

`schemaVersion` gates the manifest shape; `apiVersion` gates the runtime
contract. A manifest with an unknown schema version is **Incompatible**, not
silently accepted. Version `1.0.0` is the only Plugin API version in this phase.
