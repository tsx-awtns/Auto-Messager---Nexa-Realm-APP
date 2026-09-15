// Nexus Realm
// File: version.rs
// Purpose: Plugin API, schema, and package version identifiers

use crate::error::ContractError;

pub const PLUGIN_API_MAJOR: u32 = 1;
pub const PLUGIN_API_MINOR: u32 = 0;
pub const PLUGIN_API_PATCH: u32 = 0;
pub const PLUGIN_API_VERSION: &str = "1.0.0";
pub const MANIFEST_SCHEMA_VERSION: u32 = 1;
pub const PACKAGE_SCHEMA_VERSION: u32 = 1;

pub fn api_version_word() -> u32 {
    (PLUGIN_API_MAJOR << 16) | (PLUGIN_API_MINOR << 8) | PLUGIN_API_PATCH
}

pub fn api_major(word: u32) -> u32 {
    word >> 16
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ApiVersion {
    major: u32,
    minor: u32,
    patch: u32,
}

impl ApiVersion {
    pub const CURRENT: Self = Self {
        major: PLUGIN_API_MAJOR,
        minor: PLUGIN_API_MINOR,
        patch: PLUGIN_API_PATCH,
    };

    pub fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self {
            major,
            minor,
            patch,
        }
    }

    pub fn parse(value: &str) -> Result<Self, ContractError> {
        if value.len() > 64 {
            return Err(ContractError::InvalidVersion);
        }
        let mut parts = value.split('.');
        let major = parse_component(parts.next())?;
        let minor = parse_component(parts.next())?;
        let patch = parse_component(parts.next())?;
        if parts.next().is_some() {
            return Err(ContractError::InvalidVersion);
        }
        Ok(Self {
            major,
            minor,
            patch,
        })
    }

    pub fn from_word(word: u32) -> Self {
        Self {
            major: (word >> 16) & 0xff,
            minor: (word >> 8) & 0xff,
            patch: word & 0xff,
        }
    }

    pub fn major(self) -> u32 {
        self.major
    }

    pub fn minor(self) -> u32 {
        self.minor
    }

    pub fn patch(self) -> u32 {
        self.patch
    }

    pub fn to_word(self) -> u32 {
        ((self.major & 0xff) << 16) | ((self.minor & 0xff) << 8) | (self.patch & 0xff)
    }

    pub fn is_supported(self) -> bool {
        self.major == PLUGIN_API_MAJOR
    }
}

fn parse_component(component: Option<&str>) -> Result<u32, ContractError> {
    let component = component.ok_or(ContractError::InvalidVersion)?;
    if component.is_empty() || component.len() > 9 || !component.bytes().all(|byte| byte.is_ascii_digit())
    {
        return Err(ContractError::InvalidVersion);
    }
    if component.len() > 1 && component.starts_with('0') {
        return Err(ContractError::InvalidVersion);
    }
    component
        .parse::<u32>()
        .map_err(|_| ContractError::InvalidVersion)
}

impl std::fmt::Display for ApiVersion {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}
