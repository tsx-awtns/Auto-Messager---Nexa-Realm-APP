// Nexus Realm
// File: permissions.rs
// Purpose: Closed Plugin API V1 permission vocabulary

use crate::error::ContractError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Permission {
    #[serde(rename = "app.read")]
    AppRead,
    #[serde(rename = "events.subscribe")]
    EventsSubscribe,
    #[serde(rename = "storage.plugin.read")]
    StorageRead,
    #[serde(rename = "storage.plugin.write")]
    StorageWrite,
    #[serde(rename = "notifications.show")]
    NotificationsShow,
    #[serde(rename = "network.http")]
    NetworkHttp,
    #[serde(rename = "ui.page.register")]
    UiPageRegister,
    #[serde(rename = "ui.widget.register")]
    UiWidgetRegister,
    #[serde(rename = "automation.read")]
    AutomationRead,
    #[serde(rename = "automation.control.start")]
    AutomationStart,
    #[serde(rename = "automation.control.stop")]
    AutomationStop,
    #[serde(rename = "logs.write")]
    LogsWrite,
}

impl Permission {
    pub const ALL: [Self; 12] = [
        Self::AppRead,
        Self::EventsSubscribe,
        Self::StorageRead,
        Self::StorageWrite,
        Self::NotificationsShow,
        Self::NetworkHttp,
        Self::UiPageRegister,
        Self::UiWidgetRegister,
        Self::AutomationRead,
        Self::AutomationStart,
        Self::AutomationStop,
        Self::LogsWrite,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::AppRead => "app.read",
            Self::EventsSubscribe => "events.subscribe",
            Self::StorageRead => "storage.plugin.read",
            Self::StorageWrite => "storage.plugin.write",
            Self::NotificationsShow => "notifications.show",
            Self::NetworkHttp => "network.http",
            Self::UiPageRegister => "ui.page.register",
            Self::UiWidgetRegister => "ui.widget.register",
            Self::AutomationRead => "automation.read",
            Self::AutomationStart => "automation.control.start",
            Self::AutomationStop => "automation.control.stop",
            Self::LogsWrite => "logs.write",
        }
    }

    pub fn parse(value: &str) -> Result<Self, ContractError> {
        Self::ALL
            .into_iter()
            .find(|permission| permission.as_str() == value)
            .ok_or(ContractError::InvalidPermission)
    }
}

pub const PROTECTED_CAPABILITIES: [&str; 11] = [
    "core.modify",
    "security.modify",
    "integrity.modify",
    "root_trust.modify",
    "trusted_keys.modify",
    "updater.modify",
    "credentials.raw",
    "dpapi.raw",
    "process.execute",
    "shell.execute",
    "filesystem.global",
];

pub fn validate_grants(
    requested: &[Permission],
    granted: &[Permission],
) -> Result<(), ContractError> {
    let unique: std::collections::BTreeSet<_> = granted.iter().collect();
    if granted.len() > requested.len()
        || unique.len() != granted.len()
        || granted
            .iter()
            .any(|permission| !requested.contains(permission))
    {
        return Err(ContractError::PermissionDenied);
    }
    Ok(())
}
