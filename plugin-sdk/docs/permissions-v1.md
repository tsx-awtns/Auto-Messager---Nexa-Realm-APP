# Permissions V1

Permissions describe what a plugin may *ask* Nexus Realm to do. They are
requested in `plugin.json`, granted by Nexus Realm, and never granted by the
plugin itself.

## Grantable identifiers

| Identifier | Intent |
| --- | --- |
| `app.read` | read public application information |
| `events.subscribe` | receive public events |
| `storage.plugin.read` | read the plugin's own storage |
| `storage.plugin.write` | write the plugin's own storage |
| `notifications.show` | show user-visible notifications |
| `network.http` | perform HTTP requests |
| `ui.page.register` | register a plugin page |
| `ui.widget.register` | register a plugin widget |
| `automation.read` | read automation state |
| `automation.control.start` | start automation |
| `automation.control.stop` | stop automation |
| `logs.write` | write to the plugin log |

Declaring a permission does not implement it. Only `logs.write` currently backs
a host capability: it is required by the bounded logging Host API. The other
identifiers are declared contracts with no implementation yet, and an
unauthorised host call is denied at runtime.

## Protected capabilities

These identifiers exist only to document what can never be requested. They are
not part of the permission enum and are rejected by the validator:

```
core.modify  security.modify  integrity.modify  root_trust.modify
trusted_keys.modify  updater.modify  credentials.raw  dpapi.raw
process.execute  shell.execute  filesystem.global
```

## Grants

A grant set must be a duplicate-free subset of the permissions the manifest
requests. The backend records grants; the registry discards cached grants on
load and re-derives security state, so a stored grant can never become
authorization. Package and manifest content can request permissions, but a
structurally valid package is still not trusted.

## Using permissions in Rust

```rust
PluginMetadata::new("com.example.my-plugin", "My Plugin", "1.0.0")
    .permission(Permission::EventsSubscribe)
    .subscribe(EventKind::AppReady)
```

Subscribing to an event without `Permission::EventsSubscribe` fails validation
with `permission_denied`.
