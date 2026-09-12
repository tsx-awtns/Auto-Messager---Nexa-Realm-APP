# 🌌 Nexus Realm — Auto Messager

> **Native Rust Architecture · Secure Updates · Discord Automation · Built for Windows**

Nexus Realm is the next generation of **Auto Messager**.

What originally started as the Sailor Cross / Auto Messager project has evolved into a significantly larger Windows application with a new native backend, custom installation system, secure update infrastructure, integrity verification, and a modern React interface.

The project is no longer based around an Electron application architecture.

**Nexus Realm is now powered by Rust + Tauri.**

---

## 🚀 Current Release

### Nexus Realm r.v6.2.0

r.v6.2.0 represents one of the largest architectural upgrades in the history of Auto Messager.

The application now includes:

- 🦀 Native Rust backend
- ⚡ Tauri v2 desktop runtime
- 🎨 React + TypeScript interface
- 🌐 Native WinHTTP networking
- 🔐 Root-of-Trust update security
- 🛡️ Nexus Integrity
- 🔑 DPAPI-protected sensitive storage
- 📦 Custom native installer
- 🗑️ Custom native uninstaller
- 🔄 Secure automatic updater
- 🩹 Secure repair system
- 📖 Advanced built-in Markdown changelog reader
- 🖥️ Native Windows integration

---

# 🦀 From Electron to Rust

One of the biggest changes to Nexus Realm was the migration away from the original Electron backend.

The application now uses:

```text
React + TypeScript
        │
        ▼
     Tauri v2
        │
        ▼
       Rust
        │
        ▼
 Native Windows APIs
```

The existing frontend experience was preserved while major backend responsibilities were moved into native Rust components.

This includes areas such as:

- networking
- update handling
- filesystem operations
- secure storage
- process management
- integrity verification
- installation
- uninstallation
- repair
- native Windows integration

This reduces the application's dependency on the old Electron/Node.js runtime and provides a stronger foundation for future development.

---

# ⚡ Native Windows Architecture

Nexus Realm is designed primarily for Windows.

Important operations are handled through native components wherever practical.

Examples include:

- Native Rust backend
- Windows DPAPI
- WinHTTP
- Native process handling
- Native external-link handling
- Native installer
- Native uninstaller
- Native cleanup helper
- Native update launcher
- Windows Registry integration
- Native single-instance handling
- System tray integration

No PowerShell or CMD scripts are required for the normal update process.

---

# 🔐 Root-of-Trust Security

Nexus Realm uses a multi-level trust architecture for software updates.

Instead of permanently trusting one release signing key, the application contains a long-lived Root-of-Trust.

```text
ROOT PUBLIC KEY
       │
       ▼
trusted-keys.json.sig
       │
       ▼
trusted-keys.json
       │
       ▼
Authorized Release Key
       │
       ▼
release-index.json.sig
       │
       ▼
release-index.json
       │
       ▼
Installer SHA-256 + Size
       │
       ▼
Verified Setup
       │
       ▼
Installation
```

The Root key authorizes which release signing keys Nexus Realm is allowed to trust.

Release keys can therefore be:

- Active
- Retired
- Revoked

This allows future signing-key rotation without replacing the permanent Root-of-Trust.

---

# 🛡️ Nexus Integrity

Nexus Realm contains its own application-integrity verification system.

Protected application files are recorded in a signed integrity manifest.

The system verifies information such as:

- file paths
- expected files
- file sizes
- SHA-256 hashes
- protected directories
- signed integrity metadata

Sensitive functionality can be restricted when application integrity cannot be verified.

The integrity system is designed to fail closed instead of silently ignoring verification failures.

---

# 🔄 Secure Auto-Updater

Nexus Realm contains a built-in native updater.

The updater can:

1. Detect a newer release
2. Load the trusted release-key metadata
3. Verify the Root signature
4. Resolve the authorized signing key
5. Verify the signed release index
6. Verify release identity
7. Download the installer
8. Verify installer size
9. Verify SHA-256
10. Launch the verified installer

An installer is never intentionally launched when required security verification fails.

There is no:

- `Install Anyway`
- unsigned update fallback
- signature bypass

If an automatic update cannot be verified safely, Nexus Realm directs the user to the official release page instead.

---

# 🔑 Trusted Release Keys

Release signing keys are managed through a Root-signed trusted keyring.

The keyring supports:

```text
active
retired
revoked
```

It also supports:

- Signing Key IDs
- validity windows
- keyring generations
- rollback protection
- future release-key rotation

The updater only accepts an appropriate active key for a new release.

Revoked keys are rejected.

---

# 🩹 Secure Repair System

The Repair system follows the same Root-of-Trust architecture.

Its verification chain is:

```text
Compiled Root Key
        ↓
Installed Trusted Keyring
        ↓
Signing Key ID
        ↓
Authorized Release Key
        ↓
Signed Recovery / Release Metadata
        ↓
Verified Installer
        ↓
Repair
```

Historical repair is handled separately from installing a new update.

A retired key may remain valid for legitimate historical recovery where policy allows it.

A revoked key is rejected.

---

# 📦 Native Installer

Nexus Realm includes its own installer instead of relying on the old generic installation architecture.

The installer supports:

- Fresh installation
- Updating
- Reinstallation
- Downgrading
- Repair scenarios
- Custom installation paths
- Existing-install detection
- Registry integration
- Shortcut management
- Process handling
- Transactional installation behavior

Installation uses staging and backup mechanisms to reduce the risk of leaving the application in a partially installed state.

---

# ♻️ Transactional Updates

Application files are not simply overwritten blindly.

The installer uses a staged transaction model similar to:

```text
New Payload
    ↓
Staging Directory
    ↓
Verify
    ↓
Current Installation → Backup
    ↓
Staging → Active Installation
    ↓
Final Verification
    ↓
Remove Backup
```

If an installation operation fails at the wrong point, rollback mechanisms can restore the previous installation where possible.

---

# 🗑️ Native Uninstaller

The uninstaller has also been rewritten.

Nexus Realm now uses:

- Native Rust uninstaller
- Native cleanup helper
- Custom Nexus Realm UI
- Background cleanup
- Retry-based directory removal

The old Electron-based uninstallation runtime is no longer required.

---

# 🔒 Secure Local Storage

Sensitive application data is stored locally and protected using Windows security functionality.

Nexus Realm uses **Windows DPAPI** for protected sensitive values such as stored authentication data.

This means sensitive values can be encrypted using protection tied to the Windows user environment rather than relying on a static application encryption key.

Legacy data can be migrated where supported.

---

# 🌐 Native Networking

Important backend networking has been moved to native Rust networking built around **Windows WinHTTP**.

This is used for areas including:

- update checks
- release metadata
- changelog retrieval
- installer downloads
- Discord transport

Network operations include protections such as:

- bounded timeouts
- response-size limits
- redirect handling
- typed errors
- trusted-host restrictions where required

---

# 💬 Discord Transport

Nexus Realm includes native Discord transport functionality used by the Auto Messager system.

The transport layer includes:

- request validation
- native HTTP requests
- cancellation handling
- typed failures
- response parsing
- duplicate protection
- integration with Nexus Integrity

The application remains under active development and additional automation functionality is planned.

---

# 📖 Advanced Changelog Reader

Nexus Realm contains a built-in Markdown changelog viewer.

The current renderer supports a broad GitHub-style Markdown subset including:

- H1–H6 headings
- **Bold**
- *Italic*
- ***Bold + Italic***
- ~~Strikethrough~~
- Inline code
- Fenced code blocks
- Indented code blocks
- Ordered lists
- Unordered lists
- Nested lists
- Task lists
- Blockquotes
- Tables
- Table alignment
- Horizontal rules
- Links
- Autolinks
- Responsive images
- Collapsible details sections
- HTML entities
- Unicode
- Emoji
- Escaped Markdown characters

The renderer treats remote changelog content as untrusted input.

It does not directly inject arbitrary Markdown HTML into the application DOM.

---

# 🔗 Safe External Links

External links from supported application content are routed through controlled link handling.

Unsafe protocols and executable URL schemes are rejected.

This prevents changelog content from simply becoming an unrestricted navigation or script-execution surface.

---

# 🖥️ Desktop Experience

Nexus Realm retains its custom desktop experience while running on the newer Tauri/Rust backend.

Features include:

- Frameless application window
- Custom title bar
- Native minimize/maximize/close integration
- System tray
- Single-instance protection
- Existing-instance focus handling
- Custom Nexus Realm styling
- Responsive interface

---

# 🏗️ Project Architecture

A simplified view of the repository:

```text
Nexus Realm
│
├── src/
│   └── React + TypeScript frontend
│
├── main-interface/
│   └── Rust + Tauri main application
│
├── installer/
│   └── Native Rust/Tauri installer
│
├── uninstaller/
│   └── Native Rust uninstaller
│
├── scripts/
│   └── Build, packaging and security tooling
│
└── CHANGELOG.md
    └── Release history
```

---

# 🏭 Release Pipeline

Production releases are built through a centralized Windows build pipeline.

A simplified flow:

```text
Clean
  ↓
Version Sync
  ↓
Build Stamp
  ↓
TypeScript Validation
  ↓
Icons
  ↓
Native Uninstaller
  ↓
Frontend
  ↓
Security Identity
  ↓
Native Tauri Application
  ↓
Trusted Key Metadata
  ↓
Nexus Integrity
  ↓
Native Installer
  ↓
Signed Release Metadata
```

Release builds perform consistency and security checks before the resulting artifacts are considered publishable.

---

# 🏷️ Centralized Version Management

The application uses one central release-version source.

```text
package.json
    ↓
Application
Rust crates
Tauri
Installer
Uninstaller
About
BUILD_ID
Setup filename
Integrity metadata
Release metadata
```

This reduces mismatched version information between different application components.

---

# 🔨 Technology

Nexus Realm currently uses technologies including:

| Area | Technology |
| --- | --- |
| Desktop Runtime | Tauri v2 |
| Native Backend | Rust |
| Frontend | React |
| Frontend Language | TypeScript |
| Build Frontend | Vite |
| Windows Networking | WinHTTP |
| Secure Storage | Windows DPAPI |
| Integrity Hashing | SHA-256 |
| Signatures | Ed25519 |
| Platform | Windows |

---

# 🪟 Platform Support

### Windows

✅ **Primary and actively supported platform**

Nexus Realm is currently designed around Windows and makes use of Windows-specific native functionality.

### Linux

❌ No active support.

### macOS

❌ Not currently supported.

---

# ⚠️ Important Migration Notice

Older Auto Messager / Nexus Realm installations from before the current Root-of-Trust architecture may not be able to automatically verify the transition to the newer signing infrastructure.

This is intentional fail-closed behavior.

If an older installation reports that an update cannot be completed safely, perform a one-time manual migration:

1. Uninstall the old application.
2. Download the current installer from the official Nexus Realm repository.
3. Install the current version.

Saved application data does not normally need to be intentionally deleted for this migration.

After moving to the new Root-of-Trust architecture, future authorized signing-key rotations can be handled by the new trust system.

---

# 📥 Download

The latest official releases are available from the Nexus Realm GitHub repository:

**GitHub Releases:**  
https://github.com/tsx-awtns/Auto-Messager---Nexa-Realm-APP/releases

Download the latest:

```text
Auto.Messager-Setup-x.x.x.exe
```

and run the installer.

The built-in updater handles supported future updates automatically.

---

# 📝 Changelog

Detailed release information is available in:

```text
CHANGELOG.md
```

The same changelog can also be viewed directly inside Nexus Realm.

---

# 💬 Support

Found a bug or have a problem?

Join the official Discord server:

https://discord.gg/pwGwGkx8NC

Please provide useful information when reporting problems, such as:

- Nexus Realm version
- What you were doing
- What you expected to happen
- What actually happened
- Relevant error message
- Reproduction steps

Do not publicly post authentication tokens or other sensitive information.

---

# 👨‍💻 Development

Nexus Realm / Auto Messager is developed and maintained by:

**noura29x**

Full-stack & solo developer.

---

# 🕰️ Project History

The original Auto Messager project existed before Nexus Realm under the older **SV / v3.x** architecture.

After the original project/code became unavailable in December 2025, the application was rebuilt from scratch as the Sailor Cross generation.

That rewrite later evolved further into **Nexus Realm**, including the migration away from the Electron backend and toward the current native Rust/Tauri architecture.

The project has therefore gone through several major generations:

```text
Auto Messager SV / v3.x
          ↓
Sailor Cross / SCv5
          ↓
Nexus Realm
          ↓
Native Rust + Tauri Architecture
```

---

# 💙 Nexus Realm

Nexus Realm is no longer just a rewrite of the old Auto Messager.

It is now the foundation for the project's future development:

**Native. Secure. Maintainable. Expandable.**

More is coming.
