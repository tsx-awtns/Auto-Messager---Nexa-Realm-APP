// Nexus Realm
// File: identity.rs
// Purpose: Validated plugin identifiers and content hashes

use crate::error::ContractError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct PluginId(String);

impl PluginId {
    pub fn parse(value: &str) -> Result<Self, ContractError> {
        if value.len() < 5 || value.len() > 128 || !value.is_ascii() {
            return Err(ContractError::InvalidId);
        }
        let parts: Vec<_> = value.split('.').collect();
        if parts.len() < 3
            || parts.iter().any(|part| {
                part.is_empty()
                    || part.len() > 63
                    || !part.as_bytes()[0].is_ascii_lowercase()
                    || !part.as_bytes()[part.len() - 1].is_ascii_alphanumeric()
                    || !part
                        .bytes()
                        .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
            })
        {
            return Err(ContractError::InvalidId);
        }
        crate::path::validate_relative(value).map_err(|_| ContractError::InvalidId)?;
        Ok(Self(value.into()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn short_name(&self) -> &str {
        self.0.rsplit('.').next().unwrap_or(&self.0)
    }
}

impl TryFrom<String> for PluginId {
    type Error = ContractError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::parse(&value)
    }
}

impl From<PluginId> for String {
    fn from(value: PluginId) -> Self {
        value.0
    }
}

impl std::fmt::Display for PluginId {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct Sha256Hash(String);

impl Sha256Hash {
    pub fn parse(value: &str) -> Result<Self, ContractError> {
        if value.len() != 64
            || !value
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        {
            return Err(ContractError::InvalidManifest);
        }
        Ok(Self(value.into()))
    }

    pub fn digest(bytes: &[u8]) -> Self {
        use sha2::{Digest, Sha256};
        Self(hex::encode(Sha256::digest(bytes)))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for Sha256Hash {
    type Error = ContractError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::parse(&value)
    }
}

impl From<Sha256Hash> for String {
    fn from(value: Sha256Hash) -> Self {
        value.0
    }
}

impl std::fmt::Display for Sha256Hash {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}
