// Nexus Realm
// File: events.rs
// Purpose: Closed Public Event Contract V1 signals

use serde::{Deserialize, Serialize};

pub const EVENT_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum EventKind {
    #[serde(rename = "app.started")]
    AppStarted,
    #[serde(rename = "app.ready")]
    AppReady,
    #[serde(rename = "app.closing")]
    AppClosing,
    #[serde(rename = "app.focused")]
    AppFocused,
    #[serde(rename = "app.unfocused")]
    AppUnfocused,
    #[serde(rename = "window.shown")]
    WindowShown,
    #[serde(rename = "window.hidden")]
    WindowHidden,
    #[serde(rename = "network.connected")]
    NetworkConnected,
    #[serde(rename = "network.disconnected")]
    NetworkDisconnected,
    #[serde(rename = "automation.started")]
    AutomationStarted,
    #[serde(rename = "automation.paused")]
    AutomationPaused,
    #[serde(rename = "automation.resumed")]
    AutomationResumed,
    #[serde(rename = "automation.stopped")]
    AutomationStopped,
    #[serde(rename = "automation.failed")]
    AutomationFailed,
    #[serde(rename = "automation.completed")]
    AutomationCompleted,
    #[serde(rename = "account.available")]
    AccountAvailable,
    #[serde(rename = "account.unavailable")]
    AccountUnavailable,
    #[serde(rename = "account.connection_changed")]
    AccountConnectionChanged,
    #[serde(rename = "security.verification_started")]
    SecurityVerificationStarted,
    #[serde(rename = "security.verification_completed")]
    SecurityVerificationCompleted,
    #[serde(rename = "security.trusted")]
    SecurityTrusted,
    #[serde(rename = "security.untrusted")]
    SecurityUntrusted,
    #[serde(rename = "update.check_started")]
    UpdateCheckStarted,
    #[serde(rename = "update.available")]
    UpdateAvailable,
    #[serde(rename = "update.not_available")]
    UpdateNotAvailable,
    #[serde(rename = "update.download_started")]
    UpdateDownloadStarted,
    #[serde(rename = "update.download_completed")]
    UpdateDownloadCompleted,
    #[serde(rename = "update.install_started")]
    UpdateInstallStarted,
    #[serde(rename = "plugin.loaded")]
    PluginLoaded,
    #[serde(rename = "plugin.unloaded")]
    PluginUnloaded,
    #[serde(rename = "plugin.enabled")]
    PluginEnabled,
    #[serde(rename = "plugin.disabled")]
    PluginDisabled,
    #[serde(rename = "plugin.error")]
    PluginError,
}

impl EventKind {
    pub const ALL: [Self; 33] = [
        Self::AppStarted,
        Self::AppReady,
        Self::AppClosing,
        Self::AppFocused,
        Self::AppUnfocused,
        Self::WindowShown,
        Self::WindowHidden,
        Self::NetworkConnected,
        Self::NetworkDisconnected,
        Self::AutomationStarted,
        Self::AutomationPaused,
        Self::AutomationResumed,
        Self::AutomationStopped,
        Self::AutomationFailed,
        Self::AutomationCompleted,
        Self::AccountAvailable,
        Self::AccountUnavailable,
        Self::AccountConnectionChanged,
        Self::SecurityVerificationStarted,
        Self::SecurityVerificationCompleted,
        Self::SecurityTrusted,
        Self::SecurityUntrusted,
        Self::UpdateCheckStarted,
        Self::UpdateAvailable,
        Self::UpdateNotAvailable,
        Self::UpdateDownloadStarted,
        Self::UpdateDownloadCompleted,
        Self::UpdateInstallStarted,
        Self::PluginLoaded,
        Self::PluginUnloaded,
        Self::PluginEnabled,
        Self::PluginDisabled,
        Self::PluginError,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::AppStarted => "app.started",
            Self::AppReady => "app.ready",
            Self::AppClosing => "app.closing",
            Self::AppFocused => "app.focused",
            Self::AppUnfocused => "app.unfocused",
            Self::WindowShown => "window.shown",
            Self::WindowHidden => "window.hidden",
            Self::NetworkConnected => "network.connected",
            Self::NetworkDisconnected => "network.disconnected",
            Self::AutomationStarted => "automation.started",
            Self::AutomationPaused => "automation.paused",
            Self::AutomationResumed => "automation.resumed",
            Self::AutomationStopped => "automation.stopped",
            Self::AutomationFailed => "automation.failed",
            Self::AutomationCompleted => "automation.completed",
            Self::AccountAvailable => "account.available",
            Self::AccountUnavailable => "account.unavailable",
            Self::AccountConnectionChanged => "account.connection_changed",
            Self::SecurityVerificationStarted => "security.verification_started",
            Self::SecurityVerificationCompleted => "security.verification_completed",
            Self::SecurityTrusted => "security.trusted",
            Self::SecurityUntrusted => "security.untrusted",
            Self::UpdateCheckStarted => "update.check_started",
            Self::UpdateAvailable => "update.available",
            Self::UpdateNotAvailable => "update.not_available",
            Self::UpdateDownloadStarted => "update.download_started",
            Self::UpdateDownloadCompleted => "update.download_completed",
            Self::UpdateInstallStarted => "update.install_started",
            Self::PluginLoaded => "plugin.loaded",
            Self::PluginUnloaded => "plugin.unloaded",
            Self::PluginEnabled => "plugin.enabled",
            Self::PluginDisabled => "plugin.disabled",
            Self::PluginError => "plugin.error",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|kind| kind.as_str() == value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PublicEvent {
    pub schema_version: u32,
    pub sequence: u64,
    pub occurred_at_ms: u64,
    pub kind: EventKind,
}
