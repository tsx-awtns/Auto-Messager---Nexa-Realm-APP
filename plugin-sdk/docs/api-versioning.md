# API versioning

Four version axes move independently. Do not tie them together.

| Axis | Value today | Where it lives | Changes when |
| --- | --- | --- | --- |
| Nexus Realm application version | `6.2.0` | the application package | the product ships a release |
| Plugin API version | `1.0.0` | `nexus-plugin-api`, contract and ABI | the public plugin contract changes |
| Manifest schema version | `1` | the manifest `schemaVersion` field | the manifest shape changes |
| Package schema version | `1` | `package-integrity.json` | the package metadata shape changes |
| SDK crate version | `0.1.0` | the SDK/CLI crates | SDK ergonomics or tooling change |

The Plugin API version is authoritative for compatibility. It is expressed as a
string (`"1.0.0"`) for humans and as one `u32` word for the ABI
`(major << 16 | minor << 8 | patch)`.

## Compatibility rules

- A different API **major** version is refused: a host must not load a plugin
  built for an unknown major, and a plugin must not claim one.
- Manifest schema, package schema, and event schema versions are checked exactly.
- The SDK crate version is not a compatibility signal. Building a plugin with SDK
  `0.1.x` produces a Plugin API `1.0.0` module.

## Future evolution

Additive changes inside a major version keep existing plugins valid: new
permissions, new event identifiers, new optional fields with defaults, and new
exports that older hosts can ignore. Removing or redefining an identifier
requires a new API major version, and every public vocabulary is closed, so an
unknown value is rejected rather than ignored.
