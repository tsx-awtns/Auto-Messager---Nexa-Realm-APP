// Nexus Realm
// File: integrity.rs
// Purpose: Package integrity metadata construction and verification

use crate::archive::Entry;
use crate::error::PackageError;
use nexus_plugin_api::{PackageFile, PackageIntegrity, Sha256Hash, PACKAGE_INTEGRITY_FILE};
use std::collections::BTreeSet;

pub fn package_files(entries: &[Entry]) -> Result<Vec<PackageFile>, PackageError> {
    let mut files: Vec<PackageFile> = entries
        .iter()
        .filter(|entry| entry.name != PACKAGE_INTEGRITY_FILE)
        .map(|entry| PackageFile {
            path: entry.name.clone(),
            size: entry.bytes.len() as u64,
            sha256: Sha256Hash::digest(&entry.bytes),
        })
        .collect();
    files.sort_by(|left, right| left.path.cmp(&right.path));
    if files.is_empty() {
        return Err(PackageError::Integrity(
            "package has no content entries".to_string(),
        ));
    }
    Ok(files)
}

pub fn build(entries: &[Entry], integrity: &PackageIntegrity) -> Result<(), PackageError> {
    let files = package_files(entries)?;
    if files != integrity.files {
        return Err(PackageError::Integrity(
            "package integrity metadata does not describe the package content".to_string(),
        ));
    }
    integrity.validate().map_err(|error| {
        PackageError::Integrity(format!(
            "package integrity metadata was rejected: {}",
            error.code()
        ))
    })?;
    Ok(())
}

pub fn verify(entries: &[Entry], integrity: &PackageIntegrity) -> Result<(), PackageError> {
    integrity.validate().map_err(|error| {
        PackageError::Integrity(format!(
            "package integrity metadata was rejected: {}",
            error.code()
        ))
    })?;
    let mut recorded: BTreeSet<&str> = BTreeSet::new();
    for file in &integrity.files {
        let entry = entries
            .iter()
            .find(|entry| entry.name == file.path)
            .ok_or_else(|| {
                PackageError::Integrity(format!(
                    "package integrity metadata lists missing entry `{}`",
                    file.path
                ))
            })?;
        if entry.bytes.len() as u64 != file.size || Sha256Hash::digest(&entry.bytes) != file.sha256 {
            return Err(PackageError::Integrity(format!(
                "`{}` does not match its recorded integrity metadata",
                file.path
            )));
        }
        recorded.insert(file.path.as_str());
    }
    for entry in entries {
        if entry.name != PACKAGE_INTEGRITY_FILE && !recorded.contains(entry.name.as_str()) {
            return Err(PackageError::Integrity(format!(
                "entry `{}` is not covered by integrity metadata",
                entry.name
            )));
        }
    }
    Ok(())
}

pub fn integrity_entry(integrity: &PackageIntegrity) -> Result<Entry, PackageError> {
    Entry::new(
        PACKAGE_INTEGRITY_FILE,
        integrity.to_canonical_json().into_bytes(),
    )
}
