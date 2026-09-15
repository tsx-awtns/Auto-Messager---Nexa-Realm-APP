// Nexus Realm
// File: error.rs
// Purpose: CLI failure taxonomy with actionable messages

use nexus_plugin_api::ContractError;

#[derive(Debug)]
pub enum CliError {
    Usage(String),
    Contract(ContractError),
    Io(String),
    Project(String),
    Package(String),
    Wasm(String),
    Toolchain(String),
    Command(String),
    Conflict(String),
}

impl CliError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::Usage(_) => "usage",
            Self::Contract(_) => "contract",
            Self::Io(_) => "io",
            Self::Project(_) => "project",
            Self::Package(_) => "package",
            Self::Wasm(_) => "wasm",
            Self::Toolchain(_) => "toolchain",
            Self::Command(_) => "command",
            Self::Conflict(_) => "conflict",
        }
    }
}

impl std::fmt::Display for CliError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Usage(message) => write!(formatter, "{message}"),
            Self::Contract(error) => write!(formatter, "Plugin contract rejected the input: {}", error.code()),
            Self::Io(message) => write!(formatter, "Filesystem error: {message}"),
            Self::Project(message) => write!(formatter, "Project rejected: {message}"),
            Self::Package(message) => write!(formatter, "Package rejected: {message}"),
            Self::Wasm(message) => write!(formatter, "WebAssembly module rejected: {message}"),
            Self::Toolchain(message) => write!(formatter, "Toolchain error: {message}"),
            Self::Command(message) => write!(formatter, "Command failed: {message}"),
            Self::Conflict(message) => write!(formatter, "{message}"),
        }
    }
}

impl std::error::Error for CliError {}

impl From<ContractError> for CliError {
    fn from(error: ContractError) -> Self {
        Self::Contract(error)
    }
}

impl From<std::io::Error> for CliError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error.to_string())
    }
}

impl From<nexus_plugin_package::PackageError> for CliError {
    fn from(error: nexus_plugin_package::PackageError) -> Self {
        match error {
            nexus_plugin_package::PackageError::Module(detail) => Self::Wasm(detail),
            other => Self::Package(other.detail().to_string()),
        }
    }
}
