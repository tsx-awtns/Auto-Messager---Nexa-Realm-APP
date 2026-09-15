# Build and packaging

## `nexus-plugin build`

1. The project directory is resolved and `plugin.json` is read and fully
   validated against Manifest V1.
2. `Cargo.toml` must depend on `nexus-plugin-sdk`.
3. The target's standard library is resolved with `rustc --print target-libdir
   --target <target>`. When it is missing, the command stops with the exact
   `rustup target add <target>` instruction and never installs anything.
4. `cargo build --release --target wasm32-wasip1` runs with inherited standard
   input, output, and error, so Cargo output is never hidden.
5. The produced module is read, size-checked, and inspected: it must be a
   WebAssembly version 1 core module that exports the Plugin API v1 surface.
6. The module is staged as `<project>/plugin.wasm`.

`--offline` is forwarded to Cargo. `--target` overrides the target, which is
useful for diagnostics but the official target remains `wasm32-wasip1`.

## `nexus-plugin validate`

Structural validation of a project (`--path`) or a package (`--package`). It
checks the manifest, the id, SemVer, the API version, the entry, permissions,
subscriptions, limits, required files, the WebAssembly header and version, the
presence of the ABI exports, plugin-relative path safety, forbidden native
payloads, and — for packages — duplicate entries, unsafe entry names, CRC-32,
sizes, and SHA-256 integrity metadata.

It deliberately does not claim more than that:

```
STRUCTURAL VALIDATION: PASSED (project)
...
Structural validation only: Nexus Realm runtime security, sandboxing, and
publisher trust are NOT evaluated here, and a structurally valid plugin is not TRUSTED.
```

Nexus Realm performs its own runtime admission and sandboxing after a package is
installed. Publisher trust, revocation, consent, and real malware analysis
belong to later phases and are not simulated here.

## `nexus-plugin pack`

Packages exactly:

```
plugin.json
plugin.wasm
assets/**            (optional, in sorted order)
package-integrity.json
```

The package is written to `dist/<short-name>-<version>.nexusplugin`, where
`<short-name>` is the last label of the plugin id. Output is deterministic: the
same project produces byte-identical packages, because entry order, timestamps,
compression method, and metadata serialization are all fixed.

There is deliberately **no** source code, `Cargo.toml`, `target/`, Cargo caches,
`.git`, executables, libraries, or scripts in a package. `pack` refuses to run
without a built `plugin.wasm`, and it never copies anything into
`UserPlugins/`.

## Publishing status

Packages are unsigned and unverified. SHA-256 integrity metadata proves content
consistency; it does not prove who produced the plugin. Publisher signatures,
revocation, and the `.nexusplugin` installer are deferred to later phases.
