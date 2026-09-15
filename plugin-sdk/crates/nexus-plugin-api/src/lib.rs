// Nexus Realm
// File: lib.rs
// Purpose: Public Nexus Realm Plugin API v1 contract

pub mod abi;
pub mod error;
pub mod events;
pub mod identity;
pub mod limits;
pub mod manifest;
pub mod package;
pub mod path;
pub mod permissions;
pub mod version;

pub use error::ContractError;
pub use events::{EventKind, PublicEvent, EVENT_SCHEMA_VERSION};
pub use identity::{PluginId, Sha256Hash};
pub use limits::{
    forbidden_suffix_of, is_forbidden_suffix, ASSETS_DIR, FORBIDDEN_SUFFIXES, MANIFEST_FILE,
    MAX_MANIFEST_BYTES, MAX_MODULE_BYTES, MAX_PERMISSIONS, MAX_PLUGIN_BYTES, MAX_PLUGIN_DEPTH,
    MAX_PLUGIN_FILES, MAX_SUBSCRIPTIONS, MODULE_FILE,
};
pub use manifest::PluginManifest;
pub use package::{PackageFile, PackageIntegrity, PACKAGE_INTEGRITY_FILE};
pub use permissions::{Permission, PROTECTED_CAPABILITIES};
pub use version::{
    api_major, api_version_word, ApiVersion, MANIFEST_SCHEMA_VERSION, PACKAGE_SCHEMA_VERSION,
    PLUGIN_API_VERSION,
};
