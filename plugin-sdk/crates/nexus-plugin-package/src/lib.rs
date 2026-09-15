// Nexus Realm
// File: lib.rs
// Purpose: Shared .nexusplugin container format and package validation

pub mod archive;
pub mod content;
pub mod error;
pub mod integrity;
pub mod wasm;

pub use archive::{
    crc32, is_allowed_entry, read_archive, validate_entry_name, write_archive, Entry, EXTENSION,
    STORE_METHOD,
};
pub use content::{build_package, read_package, BuiltPackage, PackageContent, PackageInput};
pub use error::PackageError;
pub use integrity::{build, integrity_entry, package_files, verify};
pub use wasm::{inspect, WasmInspection};
