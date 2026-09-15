// Nexus Realm
// File: limits.rs
// Purpose: Authoritative plugin content names, sizes, and forbidden payload policy

pub const MANIFEST_FILE: &str = "plugin.json";
pub const MODULE_FILE: &str = "plugin.wasm";
pub const ASSETS_DIR: &str = "assets";

pub const MAX_MANIFEST_BYTES: u64 = 16 * 1024;
pub const MAX_PERMISSIONS: usize = 12;
pub const MAX_SUBSCRIPTIONS: usize = crate::events::EventKind::ALL.len();
pub const MAX_MODULE_BYTES: u64 = 16 * 1024 * 1024;
pub const MAX_PLUGIN_BYTES: u64 = 64 * 1024 * 1024;
pub const MAX_PLUGIN_FILES: usize = 512;
pub const MAX_PLUGIN_DEPTH: usize = 8;
pub const MAX_PACKAGE_ENTRIES: usize = MAX_PLUGIN_FILES + 1;

pub const FORBIDDEN_SUFFIXES: [&str; 10] = [
    "exe", "dll", "com", "bat", "cmd", "ps1", "vbs", "sys", "msi", "scr",
];

pub fn is_forbidden_suffix(suffix: &str) -> bool {
    FORBIDDEN_SUFFIXES.contains(&suffix)
}

pub fn forbidden_suffix_of(relative: &str) -> Option<&str> {
    let suffix = relative.rsplit('.').next()?;
    if suffix.len() == relative.len() {
        return None;
    }
    let lowered = suffix.to_ascii_lowercase();
    if is_forbidden_suffix(lowered.as_str()) {
        Some(suffix)
    } else {
        None
    }
}
