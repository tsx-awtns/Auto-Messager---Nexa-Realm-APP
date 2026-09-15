// Nexus Realm
// File: error.rs
// Purpose: Typed package rejection reasons shared by the CLI and Nexus Realm

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PackageError {
    Archive(String),
    Entry(String),
    Integrity(String),
    Manifest(String),
    Module(String),
    Limit(String),
}

impl PackageError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::Archive(_) => "archive",
            Self::Entry(_) => "entry",
            Self::Integrity(_) => "integrity",
            Self::Manifest(_) => "manifest",
            Self::Module(_) => "module",
            Self::Limit(_) => "limit",
        }
    }

    pub fn detail(&self) -> &str {
        match self {
            Self::Archive(detail)
            | Self::Entry(detail)
            | Self::Integrity(detail)
            | Self::Manifest(detail)
            | Self::Module(detail)
            | Self::Limit(detail) => detail,
        }
    }
}

impl std::fmt::Display for PackageError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}: {}", self.code(), self.detail())
    }
}

impl std::error::Error for PackageError {}
