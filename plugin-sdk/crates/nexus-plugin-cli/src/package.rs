// Nexus Realm
// File: package.rs
// Purpose: Re-export of the shared .nexusplugin container implementation

pub use nexus_plugin_package::archive::{
    crc32, is_allowed_entry, read_archive, validate_entry_name, write_archive, Entry, EXTENSION,
    STORE_METHOD,
};
