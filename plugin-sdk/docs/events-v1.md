# Public Event Contract V1

Public events are typed, sanitized signals. Every event carries exactly four
fields and nothing else:

```json
{ "schemaVersion": 1, "sequence": 42, "occurredAtMs": 1789301904707, "kind": "app.ready" }
```

There is no payload map, no message string, no account data, and no security
internals. Tokens, API keys, credentials, authorization headers, DPAPI blobs,
private keys, and raw security state are not representable in an event. A
message that contains extra fields is rejected as malformed.

## Identifiers

| Group | Identifiers |
| --- | --- |
| App | `app.started`, `app.ready`, `app.closing`, `app.focused`, `app.unfocused` |
| Window | `window.shown`, `window.hidden` |
| Network | `network.connected`, `network.disconnected` |
| Automation | `automation.started`, `automation.paused`, `automation.resumed`, `automation.stopped`, `automation.failed`, `automation.completed` |
| Account | `account.available`, `account.unavailable`, `account.connection_changed` |
| Security | `security.verification_started`, `security.verification_completed`, `security.trusted`, `security.untrusted` |
| Update | `update.check_started`, `update.available`, `update.not_available`, `update.download_started`, `update.download_completed`, `update.install_started` |
| Plugin | `plugin.loaded`, `plugin.unloaded`, `plugin.enabled`, `plugin.disabled`, `plugin.error` |

33 identifiers in total, in `EventKind::ALL`.

## Subscriptions

A plugin lists the events it wants in `plugin.json`. Subscribing requires the
`events.subscribe` permission, and a subscription list is limited to the size of
the vocabulary with no duplicates.

## Handling events in Rust

```rust
fn on_event(&mut self, _context: &PluginContext, event: PluginEvent) -> PluginResult<()> {
    match event.kind() {
        EventKind::AppReady => Ok(()),
        EventKind::AutomationStarted => Ok(()),
        _ => Ok(()),
    }
}
```

Backend delivery is bounded: Nexus Realm keeps a small queue per subscriber and
drops the newest event when a subscriber falls behind instead of blocking the
application or another plugin. A running plugin receives only the events listed
in its manifest subscriptions, and each delivery runs inside the plugin's fuel
and time budget.
