// Nexus Realm
// File: error.rs
// Purpose: Public contract rejection codes shared by the application and the SDK

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContractError {
    InvalidManifest,
    UnsupportedSchema,
    UnsupportedApi,
    InvalidId,
    InvalidVersion,
    InvalidPermission,
    InvalidSubscription,
    PermissionDenied,
    PathPolicy,
    LimitExceeded,
}

impl ContractError {
    pub fn code(self) -> &'static str {
        match self {
            Self::InvalidManifest => "invalid_manifest",
            Self::UnsupportedSchema => "unsupported_schema",
            Self::UnsupportedApi => "unsupported_api",
            Self::InvalidId => "invalid_id",
            Self::InvalidVersion => "invalid_version",
            Self::InvalidPermission => "invalid_permission",
            Self::InvalidSubscription => "invalid_subscription",
            Self::PermissionDenied => "permission_denied",
            Self::PathPolicy => "path_policy",
            Self::LimitExceeded => "limit_exceeded",
        }
    }
}

impl std::fmt::Display for ContractError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "Public plugin contract rejected: {self:?}")
    }
}

impl std::error::Error for ContractError {}
