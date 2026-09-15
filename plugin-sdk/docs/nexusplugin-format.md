# `.nexusplugin` package format

A `.nexusplugin` file is a ZIP container read by the Nexus Realm Plugin
Installer phase and written by `nexus-plugin pack`.

## Container

| Property | Value |
| --- | --- |
| container | ZIP (PKZIP) |
| extension | `.nexusplugin` |
| compression | none (`STORE`, method 0) — deterministic and inspectable |
| timestamps | fixed (1980-01-01 00:00:00), so packaging is reproducible |
| names | UTF-8 with the UTF-8 flag set; ASCII by path policy |
| extra fields, comments | none |
| directory entries | not used; directories are implied by file paths |
| entry order | `plugin.json`, `plugin.wasm`, then `assets/**` sorted by path, then `package-integrity.json` |

The container is a normal ZIP: standard tools can list and extract it. Nexus
Realm does not rely on that, it validates the structure itself.

## Required contents

| Entry | Required | Notes |
| --- | --- | --- |
| `plugin.json` | yes | Plugin Manifest V1 |
| `plugin.wasm` | yes | exactly one, WebAssembly version 1 core module, at most 16 MiB |
| `assets/**` | no | optional static files, at most 64 MiB in total |
| `package-integrity.json` | yes | integrity metadata for every other entry |

No other entry is allowed. `pack` cannot write one, and validation rejects one.

## Path rules

Every entry name must satisfy the shared plugin path policy: ASCII only, forward
slashes only, no empty component, no `.` or `..`, at most 240 bytes and 128 bytes
per component, no trailing dot or space, no characters outside
`A-Za-z0-9._-`, and no Windows device names (`CON`, `PRN`, `AUX`, `NUL`,
`CLOCK$`, `COM1`–`COM9`, `LPT1`–`LPT9`). Absolute paths, drive letters, UNC
paths, device paths, and alternate data stream syntax are rejected.

## Unsupported content

- symlink or junction entries (symbolic-link attribute bits) — rejected;
- directory entries — rejected;
- encrypted entries, data descriptors, and any compression method other than
  `STORE` — rejected;
- duplicate entry names — rejected;
- native executables and libraries (`.exe`, `.dll`, `.com`, `.bat`, `.cmd`,
  `.ps1`, `.vbs`, `.sys`, `.msi`, `.scr`) — rejected;
- missing or unmatched `package-integrity.json` — rejected.

## Limits

| Limit | Value |
| --- | --- |
| entries | at most 513 |
| total content | at most 64 MiB |
| `plugin.wasm` | at most 16 MiB |
| `plugin.json` | at most 16 KiB |
| `assets/**` nesting | at most 8 levels |

## Reading rules

Validation reads the central directory, rejects multi-disk archives and archives
without a well-formed end record, verifies each entry's CRC-32 over exactly the
declared bytes, rejects names that fail the path policy even if the writer would
never produce them, and then verifies every SHA-256 and size in
`package-integrity.json` against the actual bytes. An entry that is not covered
by the integrity metadata is rejected.
