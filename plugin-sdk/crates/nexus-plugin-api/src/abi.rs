// Nexus Realm
// File: abi.rs
// Purpose: Plugin API v1 WASM boundary protocol types and status codes

use crate::{events::EventKind, permissions::Permission};
use serde::{Deserialize, Serialize};

pub const MAX_ABI_MESSAGE_BYTES: u32 = 64 * 1024;
pub const MAX_HOST_LOG_BYTES: u32 = 1024;

pub const HOST_IMPORT_MODULE: &str = "nexus_host";
pub const HOST_LOG_FUNCTION: &str = "nexus_host_log";
pub const HOST_LOG_LEVEL_INFO: u32 = 1;
pub const HOST_LOG_LEVEL_WARN: u32 = 2;
pub const HOST_LOG_LEVEL_ERROR: u32 = 3;
pub const HOST_LOG_LEVELS: [u32; 3] = [
    HOST_LOG_LEVEL_INFO,
    HOST_LOG_LEVEL_WARN,
    HOST_LOG_LEVEL_ERROR,
];
pub const HOST_CALLS: [&str; 1] = [HOST_LOG_FUNCTION];

pub const EXPORT_API_VERSION: &str = "nexus_plugin_api_version";
pub const EXPORT_ALLOC: &str = "nexus_plugin_alloc";
pub const EXPORT_FREE: &str = "nexus_plugin_free";
pub const EXPORT_INIT: &str = "nexus_plugin_init";
pub const EXPORT_DESCRIBE: &str = "nexus_plugin_describe";
pub const EXPORT_EVENT: &str = "nexus_plugin_event";
pub const EXPORT_SHUTDOWN: &str = "nexus_plugin_shutdown";

pub const REQUIRED_EXPORTS: [&str; 7] = [
    EXPORT_API_VERSION,
    EXPORT_ALLOC,
    EXPORT_FREE,
    EXPORT_INIT,
    EXPORT_DESCRIBE,
    EXPORT_EVENT,
    EXPORT_SHUTDOWN,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum Status {
    Ok = 0,
    InvalidArgument = -1,
    InvalidState = -2,
    MalformedMessage = -3,
    LimitExceeded = -4,
    UnsupportedVersion = -5,
    PluginError = -6,
    Internal = -7,
    PermissionDenied = -8,
}

impl Status {
    pub fn code(self) -> i32 {
        self as i32
    }

    pub fn from_code(code: i32) -> Option<Self> {
        Some(match code {
            0 => Self::Ok,
            -1 => Self::InvalidArgument,
            -2 => Self::InvalidState,
            -3 => Self::MalformedMessage,
            -4 => Self::LimitExceeded,
            -5 => Self::UnsupportedVersion,
            -6 => Self::PluginError,
            -7 => Self::Internal,
            -8 => Self::PermissionDenied,
            _ => return None,
        })
    }

    pub fn is_failure(self) -> bool {
        !matches!(self, Self::Ok)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AbiInitRequest {
    pub schema_version: u32,
    pub api_version: String,
    pub plugin_id: String,
}

impl AbiInitRequest {
    pub fn validate(&self) -> Result<(), Status> {
        if self.schema_version != crate::version::MANIFEST_SCHEMA_VERSION {
            return Err(Status::UnsupportedVersion);
        }
        let version = crate::version::ApiVersion::parse(&self.api_version)
            .map_err(|_| Status::UnsupportedVersion)?;
        if !version.is_supported() {
            return Err(Status::UnsupportedVersion);
        }
        crate::identity::PluginId::parse(&self.plugin_id).map_err(|_| Status::InvalidArgument)?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AbiDescribeResponse {
    pub schema_version: u32,
    pub id: String,
    pub name: String,
    pub version: String,
    pub api_version: String,
    pub permissions: Vec<Permission>,
    pub subscriptions: Vec<EventKind>,
}

impl AbiDescribeResponse {
    pub fn validate(&self) -> Result<(), Status> {
        if self.schema_version != crate::version::MANIFEST_SCHEMA_VERSION {
            return Err(Status::UnsupportedVersion);
        }
        crate::identity::PluginId::parse(&self.id).map_err(|_| Status::InvalidArgument)?;
        if self.name.is_empty() || self.name.len() > 80 {
            return Err(Status::InvalidArgument);
        }
        crate::manifest::validate_version(&self.version).map_err(|_| Status::InvalidArgument)?;
        crate::version::ApiVersion::parse(&self.api_version)
            .map_err(|_| Status::UnsupportedVersion)?;
        if self.permissions.len() > crate::limits::MAX_PERMISSIONS
            || self.subscriptions.len() > crate::limits::MAX_SUBSCRIPTIONS
        {
            return Err(Status::LimitExceeded);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AbiEvent {
    pub schema_version: u32,
    pub sequence: u64,
    pub occurred_at_ms: u64,
    pub kind: EventKind,
}

impl AbiEvent {
    pub fn validate(&self) -> Result<(), Status> {
        if self.schema_version != crate::events::EVENT_SCHEMA_VERSION {
            return Err(Status::UnsupportedVersion);
        }
        Ok(())
    }
}
