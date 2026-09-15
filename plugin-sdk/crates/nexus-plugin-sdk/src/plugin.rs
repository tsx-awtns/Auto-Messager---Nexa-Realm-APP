// Nexus Realm
// File: plugin.rs
// Purpose: Developer-facing plugin traits, metadata, context, and events

use nexus_plugin_api::{
    events::EventKind,
    manifest::validate_version,
    identity::PluginId,
    permissions::Permission,
    version::{ApiVersion, MANIFEST_SCHEMA_VERSION, PLUGIN_API_VERSION},
    abi::{AbiEvent, AbiDescribeResponse, Status},
    ContractError, PluginManifest, MODULE_FILE,
};

pub type PluginResult<T> = Result<T, PluginError>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PluginError {
    InvalidArgument,
    InvalidState,
    MalformedMessage,
    LimitExceeded,
    UnsupportedVersion,
    PermissionDenied,
    Failed,
}

impl PluginError {
    pub fn status(self) -> Status {
        match self {
            Self::InvalidArgument => Status::InvalidArgument,
            Self::InvalidState => Status::InvalidState,
            Self::MalformedMessage => Status::MalformedMessage,
            Self::LimitExceeded => Status::LimitExceeded,
            Self::UnsupportedVersion => Status::UnsupportedVersion,
            Self::PermissionDenied => Status::PermissionDenied,
            Self::Failed => Status::PluginError,
        }
    }

    pub fn message(self) -> &'static str {
        match self {
            Self::InvalidArgument => "The plugin rejected an invalid argument.",
            Self::InvalidState => "The plugin cannot serve this call in its current state.",
            Self::MalformedMessage => "The plugin received a malformed message.",
            Self::LimitExceeded => "The plugin exceeded a documented limit.",
            Self::UnsupportedVersion => "The plugin does not support the requested API version.",
            Self::PermissionDenied => {
                "Nexus Realm denied a host call because the plugin holds no grant for it."
            }
            Self::Failed => "The plugin reported a failure.",
        }
    }
}

impl From<PluginError> for Status {
    fn from(error: PluginError) -> Self {
        error.status()
    }
}

impl From<ContractError> for PluginError {
    fn from(error: ContractError) -> Self {
        match error {
            ContractError::UnsupportedSchema | ContractError::UnsupportedApi => {
                Self::UnsupportedVersion
            }
            ContractError::LimitExceeded => Self::LimitExceeded,
            ContractError::PermissionDenied => Self::PermissionDenied,
            ContractError::PathPolicy | ContractError::InvalidManifest => Self::MalformedMessage,
            _ => Self::InvalidArgument,
        }
    }
}

impl std::fmt::Display for PluginError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.message())
    }
}

impl std::error::Error for PluginError {}

#[derive(Debug, Clone)]
pub struct PluginContext {
    plugin_id: PluginId,
    api_version: ApiVersion,
}

impl PluginContext {
    pub fn new(plugin_id: PluginId, api_version: ApiVersion) -> Self {
        Self {
            plugin_id,
            api_version,
        }
    }

    pub fn plugin_id(&self) -> &PluginId {
        &self.plugin_id
    }

    pub fn api_version(&self) -> ApiVersion {
        self.api_version
    }

    pub fn log(&self) -> PluginLogger {
        PluginLogger
    }
}

pub struct PluginLogger;

impl PluginLogger {
    pub fn info(&self, message: &str) -> PluginResult<()> {
        crate::abi::host_log(nexus_plugin_api::abi::HOST_LOG_LEVEL_INFO, message)
    }

    pub fn warn(&self, message: &str) -> PluginResult<()> {
        crate::abi::host_log(nexus_plugin_api::abi::HOST_LOG_LEVEL_WARN, message)
    }

    pub fn error(&self, message: &str) -> PluginResult<()> {
        crate::abi::host_log(nexus_plugin_api::abi::HOST_LOG_LEVEL_ERROR, message)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PluginEvent {
    schema_version: u32,
    sequence: u64,
    occurred_at_ms: u64,
    kind: EventKind,
}

impl PluginEvent {
    pub fn kind(&self) -> EventKind {
        self.kind
    }

    pub fn sequence(&self) -> u64 {
        self.sequence
    }

    pub fn occurred_at_ms(&self) -> u64 {
        self.occurred_at_ms
    }

    pub fn schema_version(&self) -> u32 {
        self.schema_version
    }

    pub fn from_message(message: AbiEvent) -> PluginResult<Self> {
        message.validate().map_err(PluginError::from)?;
        Ok(Self {
            schema_version: message.schema_version,
            sequence: message.sequence,
            occurred_at_ms: message.occurred_at_ms,
            kind: message.kind,
        })
    }
}

impl From<Status> for PluginError {
    fn from(status: Status) -> Self {
        match status {
            Status::Ok => Self::Failed,
            Status::InvalidArgument => Self::InvalidArgument,
            Status::InvalidState => Self::InvalidState,
            Status::MalformedMessage => Self::MalformedMessage,
            Status::LimitExceeded => Self::LimitExceeded,
            Status::UnsupportedVersion => Self::UnsupportedVersion,
            Status::PermissionDenied => Self::PermissionDenied,
            Status::PluginError | Status::Internal => Self::Failed,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PluginMetadata {
    id: String,
    name: String,
    version: String,
    author: String,
    description: String,
    permissions: Vec<Permission>,
    subscriptions: Vec<EventKind>,
}

impl PluginMetadata {
    pub fn new(id: &str, name: &str, version: &str) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            version: version.to_string(),
            author: String::new(),
            description: String::new(),
            permissions: Vec::new(),
            subscriptions: Vec::new(),
        }
    }

    pub fn author(mut self, author: &str) -> Self {
        self.author = author.to_string();
        self
    }

    pub fn description(mut self, description: &str) -> Self {
        self.description = description.to_string();
        self
    }

    pub fn permission(mut self, permission: Permission) -> Self {
        if !self.permissions.contains(&permission) {
            self.permissions.push(permission);
        }
        self
    }

    pub fn subscribe(mut self, kind: EventKind) -> Self {
        if !self.subscriptions.contains(&kind) {
            self.subscriptions.push(kind);
        }
        self
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn version(&self) -> &str {
        &self.version
    }

    pub fn author_name(&self) -> &str {
        &self.author
    }

    pub fn description_text(&self) -> &str {
        &self.description
    }

    pub fn permissions(&self) -> &[Permission] {
        &self.permissions
    }

    pub fn subscriptions(&self) -> &[EventKind] {
        &self.subscriptions
    }

    pub fn plugin_id(&self) -> Result<PluginId, ContractError> {
        PluginId::parse(&self.id)
    }

    pub fn validate(&self) -> Result<(), ContractError> {
        self.to_manifest().map(|_| ())
    }

    pub fn to_manifest(&self) -> Result<PluginManifest, ContractError> {
        let manifest = PluginManifest {
            schema_version: MANIFEST_SCHEMA_VERSION,
            id: PluginId::parse(&self.id)?,
            name: non_empty(&self.name, "Unnamed plugin"),
            version: {
                validate_version(&self.version)?;
                self.version.clone()
            },
            author: non_empty(&self.author, "Unspecified author"),
            description: non_empty(&self.description, "Nexus Realm plugin"),
            api_version: PLUGIN_API_VERSION.to_string(),
            entry: MODULE_FILE.to_string(),
            permissions: self.permissions.clone(),
            subscriptions: self.subscriptions.clone(),
        };
        manifest.validate()?;
        Ok(manifest)
    }

    pub fn describe_response(&self) -> Result<AbiDescribeResponse, PluginError> {
        self.validate().map_err(PluginError::from)?;
        let response = AbiDescribeResponse {
            schema_version: MANIFEST_SCHEMA_VERSION,
            id: self.id.clone(),
            name: self.name.clone(),
            version: self.version.clone(),
            api_version: PLUGIN_API_VERSION.to_string(),
            permissions: self.permissions.clone(),
            subscriptions: self.subscriptions.clone(),
        };
        response.validate().map_err(PluginError::from)?;
        Ok(response)
    }
}

fn non_empty(value: &str, fallback: &str) -> String {
    if value.trim().is_empty() {
        fallback.to_string()
    } else {
        value.to_string()
    }
}

pub trait NexusPlugin: Send + Default + 'static {
    fn metadata(&self) -> PluginMetadata;

    fn on_load(&mut self, _context: &PluginContext) -> PluginResult<()> {
        Ok(())
    }

    fn on_event(&mut self, _context: &PluginContext, _event: PluginEvent) -> PluginResult<()> {
        Ok(())
    }
}
