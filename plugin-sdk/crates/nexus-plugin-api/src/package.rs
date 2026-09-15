// Nexus Realm
// File: package.rs
// Purpose: .nexusplugin package integrity model and canonical serialization

use crate::{
    error::ContractError,
    identity::{PluginId, Sha256Hash},
    limits::{
        forbidden_suffix_of, MAX_MODULE_BYTES, MAX_PACKAGE_ENTRIES, MAX_PLUGIN_BYTES,
    },
    path::{self, is_plugin_file},
    version::{ApiVersion, PACKAGE_SCHEMA_VERSION, PLUGIN_API_VERSION},
};
use serde::{Deserialize, Serialize};

pub const PACKAGE_INTEGRITY_FILE: &str = "package-integrity.json";
pub const PACKAGE_EXTENSION: &str = "nexusplugin";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PackageFile {
    pub path: String,
    pub size: u64,
    pub sha256: Sha256Hash,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PackageIntegrity {
    pub schema_version: u32,
    pub plugin_id: PluginId,
    pub plugin_version: String,
    pub api_version: String,
    pub authenticity: Authenticity,
    pub files: Vec<PackageFile>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Authenticity {
    Unsigned,
}

impl PackageIntegrity {
    pub fn new(
        plugin_id: PluginId,
        plugin_version: String,
        files: Vec<PackageFile>,
    ) -> Result<Self, ContractError> {
        let integrity = Self {
            schema_version: PACKAGE_SCHEMA_VERSION,
            plugin_id,
            plugin_version,
            api_version: PLUGIN_API_VERSION.to_string(),
            authenticity: Authenticity::Unsigned,
            files,
        };
        integrity.validate()?;
        Ok(integrity)
    }

    pub fn validate(&self) -> Result<(), ContractError> {
        if self.schema_version != PACKAGE_SCHEMA_VERSION {
            return Err(ContractError::UnsupportedSchema);
        }
        crate::manifest::validate_version(&self.plugin_version)?;
        if self.api_version != PLUGIN_API_VERSION {
            return Err(ContractError::UnsupportedApi);
        }
        if self.files.is_empty() || self.files.len() > MAX_PACKAGE_ENTRIES {
            return Err(ContractError::LimitExceeded);
        }
        let mut previous: Option<&str> = None;
        let mut total = 0u64;
        for file in &self.files {
            path::validate_relative(&file.path)?;
            if !is_plugin_file(&file.path) {
                return Err(ContractError::PathPolicy);
            }
            if forbidden_suffix_of(&file.path).is_some() {
                return Err(ContractError::InvalidManifest);
            }
            if let Some(previous) = previous {
                if previous >= file.path.as_str() {
                    return Err(ContractError::InvalidManifest);
                }
            }
            previous = Some(file.path.as_str());
            if file.path == crate::limits::MODULE_FILE && file.size > MAX_MODULE_BYTES {
                return Err(ContractError::LimitExceeded);
            }
            total = total
                .checked_add(file.size)
                .ok_or(ContractError::LimitExceeded)?;
            if total > MAX_PLUGIN_BYTES {
                return Err(ContractError::LimitExceeded);
            }
        }
        if self.files.iter().filter(|file| file.path == crate::limits::MODULE_FILE).count() != 1 {
            return Err(ContractError::InvalidManifest);
        }
        if self
            .files
            .iter()
            .filter(|file| file.path == crate::limits::MANIFEST_FILE)
            .count()
            != 1
        {
            return Err(ContractError::InvalidManifest);
        }
        Ok(())
    }

    pub fn file(&self, relative: &str) -> Option<&PackageFile> {
        self.files.iter().find(|file| file.path == relative)
    }

    pub fn verify_bytes(&self, relative: &str, bytes: &[u8]) -> Result<(), ContractError> {
        let recorded = self.file(relative).ok_or(ContractError::InvalidManifest)?;
        if recorded.size != bytes.len() as u64
            || recorded.sha256 != Sha256Hash::digest(bytes)
        {
            return Err(ContractError::InvalidManifest);
        }
        Ok(())
    }

    pub fn api_version(&self) -> Result<ApiVersion, ContractError> {
        ApiVersion::parse(&self.api_version)
    }

    pub fn parse(bytes: &[u8]) -> Result<Self, ContractError> {
        let integrity: Self = serde_json::from_slice(bytes)
            .map_err(|_| ContractError::InvalidManifest)?;
        integrity.validate()?;
        Ok(integrity)
    }

    pub fn to_canonical_json(&self) -> String {
        let mut out = String::with_capacity(256 + self.files.len() * 128);
        out.push('{');
        out.push_str("\"schemaVersion\":");
        out.push_str(&self.schema_version.to_string());
        out.push_str(",\"pluginId\":");
        push_json_string(&mut out, self.plugin_id.as_str());
        out.push_str(",\"pluginVersion\":");
        push_json_string(&mut out, &self.plugin_version);
        out.push_str(",\"apiVersion\":");
        push_json_string(&mut out, &self.api_version);
        out.push_str(",\"authenticity\":\"unsigned\",\"files\":[");
        for (index, file) in self.files.iter().enumerate() {
            if index > 0 {
                out.push(',');
            }
            out.push_str("{\"path\":");
            push_json_string(&mut out, &file.path);
            out.push_str(",\"size\":");
            out.push_str(&file.size.to_string());
            out.push_str(",\"sha256\":");
            push_json_string(&mut out, file.sha256.as_str());
            out.push('}');
        }
        out.push_str("]}");
        out
    }
}

pub fn push_json_string(out: &mut String, value: &str) {
    out.push('"');
    for character in value.chars() {
        match character {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            character if (character as u32) < 0x20 => {
                out.push_str(&format!("\\u{:04x}", character as u32));
            }
            character => out.push(character),
        }
    }
    out.push('"');
}
