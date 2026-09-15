// Nexus Realm
// File: path.rs
// Purpose: Authoritative plugin-relative path syntax policy

use crate::error::ContractError;

pub const MAX_RELATIVE_BYTES: usize = 240;
pub const MAX_COMPONENT_BYTES: usize = 128;

pub const RESERVED_DEVICE_NAMES: [&str; 5] = ["con", "prn", "aux", "nul", "clock$"];

pub fn validate_relative(relative: &str) -> Result<(), ContractError> {
    if relative.is_empty()
        || relative.len() > MAX_RELATIVE_BYTES
        || !relative.is_ascii()
        || relative.contains('\\')
    {
        return Err(ContractError::PathPolicy);
    }
    for part in relative.split('/') {
        if part.is_empty()
            || part == "."
            || part == ".."
            || part.len() > MAX_COMPONENT_BYTES
            || part.ends_with('.')
            || part.ends_with(' ')
            || !part
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || b"._-".contains(&byte))
        {
            return Err(ContractError::PathPolicy);
        }
        if is_reserved_device_name(part) {
            return Err(ContractError::PathPolicy);
        }
    }
    Ok(())
}

pub fn is_reserved_device_name(part: &str) -> bool {
    let stem = part
        .split('.')
        .next()
        .unwrap_or_default()
        .to_ascii_lowercase();
    RESERVED_DEVICE_NAMES.contains(&stem.as_str())
        || (stem.len() == 4
            && (stem.starts_with("com") || stem.starts_with("lpt"))
            && matches!(stem.as_bytes()[3], b'1'..=b'9'))
}

pub fn is_plugin_file(relative: &str) -> bool {
    matches!(relative, crate::limits::MANIFEST_FILE | crate::limits::MODULE_FILE)
        || is_assets_path(relative)
}

pub fn is_assets_path(relative: &str) -> bool {
    relative == crate::limits::ASSETS_DIR
        || relative.starts_with(&format!("{}/", crate::limits::ASSETS_DIR))
}
