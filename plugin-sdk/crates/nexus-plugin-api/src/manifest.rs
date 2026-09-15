// Nexus Realm
// File: manifest.rs
// Purpose: Strict Plugin Manifest V1 definition and compatibility checks

use crate::{
    error::ContractError,
    events::EventKind,
    identity::PluginId,
    limits::{MAX_MANIFEST_BYTES, MAX_PERMISSIONS, MAX_SUBSCRIPTIONS},
    permissions::Permission,
    version::{MANIFEST_SCHEMA_VERSION, PLUGIN_API_VERSION},
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PluginManifest {
    pub schema_version: u32,
    pub id: PluginId,
    pub name: String,
    pub version: String,
    pub author: String,
    pub description: String,
    pub api_version: String,
    pub entry: String,
    pub permissions: Vec<Permission>,
    pub subscriptions: Vec<EventKind>,
}

pub fn validate_version(value: &str) -> Result<(), ContractError> {
    if value.len() > 64 || semver::Version::parse(value).is_err() {
        return Err(ContractError::InvalidVersion);
    }
    Ok(())
}

fn valid_text(value: &str, maximum: usize) -> bool {
    !value.is_empty()
        && value.len() <= maximum
        && value.trim() == value
        && !value.chars().any(|character| character.is_control())
}

impl PluginManifest {
    pub fn parse(bytes: &[u8]) -> Result<Self, ContractError> {
        if bytes.len() as u64 > MAX_MANIFEST_BYTES {
            return Err(ContractError::LimitExceeded);
        }
        let manifest: Self =
            serde_json::from_slice(bytes).map_err(|_| ContractError::InvalidManifest)?;
        manifest.validate()?;
        Ok(manifest)
    }

    pub fn validate(&self) -> Result<(), ContractError> {
        if self.schema_version != MANIFEST_SCHEMA_VERSION {
            return Err(ContractError::UnsupportedSchema);
        }
        validate_version(&self.version)?;
        validate_version(&self.api_version)?;
        if self.api_version != PLUGIN_API_VERSION {
            return Err(ContractError::UnsupportedApi);
        }
        if !valid_text(&self.name, 80)
            || !valid_text(&self.author, 120)
            || !valid_text(&self.description, 1024)
        {
            return Err(ContractError::InvalidManifest);
        }
        crate::path::validate_relative(&self.entry).map_err(|_| ContractError::PathPolicy)?;
        if self.entry != crate::limits::MODULE_FILE {
            return Err(ContractError::InvalidManifest);
        }
        if self.permissions.len() > MAX_PERMISSIONS || self.subscriptions.len() > MAX_SUBSCRIPTIONS
        {
            return Err(ContractError::LimitExceeded);
        }
        if self.permissions.iter().collect::<BTreeSet<_>>().len() != self.permissions.len() {
            return Err(ContractError::InvalidPermission);
        }
        if self.subscriptions.iter().collect::<BTreeSet<_>>().len() != self.subscriptions.len() {
            return Err(ContractError::InvalidSubscription);
        }
        if !self.subscriptions.is_empty()
            && !self.permissions.contains(&Permission::EventsSubscribe)
        {
            return Err(ContractError::PermissionDenied);
        }
        Ok(())
    }
}
