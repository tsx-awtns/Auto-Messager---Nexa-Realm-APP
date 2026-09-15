# Security model

## Core principle

```
Plugins may extend Nexus Realm.
Plugins may never modify Nexus Realm.
```

## What the SDK cannot do

There is no API — and no plan for one — for a plugin to:

- modify Nexus Realm core files or Nexus Integrity metadata;
- modify root-of-trust material, trusted keys, or updater trust;
- read raw account credentials or DPAPI material;
- execute processes or shell commands;
- load native code, or ship a DLL/EXE/script inside a package;
- reach the filesystem globally.

There is deliberately no `execute_command`, `filesystem`, `winapi`, or
`raw_host_call` escape hatch. The only host import a plugin can resolve is the
bounded logging call, and the runtime rejects any module that imports more; a
denied call returns a status code instead of reaching the host.

## Layers that stay separate

| Layer | Owner | Question it answers |
| --- | --- | --- |
| Nexus Core Integrity | signed installation manifest + release key | is the *application* unmodified? |
| Plugin Security | plugin validation, states, quarantine | is this *plugin* acceptable? |
| Publisher trust | deferred to a later phase | who produced this plugin? |

A blocked plugin cannot change Core trust, and plugin content never becomes a
Core integrity exclusion. The user interface can show `Core Integrity: TRUSTED`
alongside `Plugin Security: 1 BLOCKED`.

## What SDK-side validation proves

`nexus-plugin validate` is structural: manifest conformance, closed vocabulary,
limits, path safety, container safety, and content hashes. It does not detect
malware, does not judge intent, and does not establish authenticity.

## Development hygiene

- Plugins are built from source in the developer's own checkout.
- `nexus-plugin build` and `pack` never write into `UserPlugins/`, so a developer
  cannot bypass Plugin Security by copying build output into an installation.
- Publisher signatures, revocation, consent, and persistent grants are deferred;
  Phase 2 packages are explicitly labeled `unsigned` in
  `package-integrity.json` and in the CLI output.

## Untrusted input handling

Everything inside a package is untrusted: sizes, names, ordering, and hashes are
all validated against the authoritative contract before use. The reader refuses
unsupported archive features instead of guessing, and rejects an entry that the
integrity metadata does not cover.
