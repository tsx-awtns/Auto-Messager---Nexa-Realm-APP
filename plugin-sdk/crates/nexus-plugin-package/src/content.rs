// Nexus Realm
// File: content.rs
// Purpose: Shared .nexusplugin reading, validation, and assembly

use crate::archive::{read_archive, validate_entry_name, Entry, STORE_METHOD};
use crate::error::PackageError;
use crate::integrity;
use crate::wasm::{self, WasmInspection};
use nexus_plugin_api::{
    PackageIntegrity, PluginManifest, ASSETS_DIR, MANIFEST_FILE, MODULE_FILE,
    PACKAGE_INTEGRITY_FILE,
};

pub struct PackageContent {
    pub manifest: PluginManifest,
    pub manifest_bytes: Vec<u8>,
    pub module_bytes: Vec<u8>,
    pub assets: Vec<(String, Vec<u8>)>,
    pub integrity: PackageIntegrity,
    pub inspection: WasmInspection,
    pub entry_names: Vec<String>,
}

pub struct PackageInput {
    pub manifest_bytes: Vec<u8>,
    pub module_bytes: Vec<u8>,
    pub assets: Vec<(String, Vec<u8>)>,
}

pub struct BuiltPackage {
    pub entries: Vec<Entry>,
    pub integrity: PackageIntegrity,
}

pub fn read_package(bytes: &[u8]) -> Result<PackageContent, PackageError> {
    let entries = read_archive(bytes)?;
    let integrity_bytes = entry(&entries, PACKAGE_INTEGRITY_FILE).ok_or_else(|| {
        PackageError::Integrity("package integrity metadata is missing".to_string())
    })?;
    let integrity = PackageIntegrity::parse(integrity_bytes).map_err(|error| {
        PackageError::Integrity(format!(
            "package integrity metadata was rejected: {}",
            error.code()
        ))
    })?;
    integrity::verify(&entries, &integrity)?;

    let manifest_bytes = entry(&entries, MANIFEST_FILE).ok_or_else(|| {
        PackageError::Manifest("package is missing `plugin.json`".to_string())
    })?;
    let manifest = PluginManifest::parse(manifest_bytes).map_err(|error| {
        PackageError::Manifest(format!("plugin manifest was rejected: {}", error.code()))
    })?;
    if integrity.plugin_id != manifest.id || integrity.plugin_version != manifest.version {
        return Err(PackageError::Integrity(
            "package integrity metadata does not match the plugin manifest".to_string(),
        ));
    }

    let module_bytes = entry(&entries, MODULE_FILE)
        .ok_or_else(|| PackageError::Module("package is missing `plugin.wasm`".to_string()))?;
    let inspection = wasm::inspect(module_bytes)?;
    let missing = inspection.missing_abi_exports();
    if !missing.is_empty() {
        return Err(PackageError::Module(format!(
            "module does not export the Plugin API v1 surface ({})",
            missing.join(", ")
        )));
    }

    let prefix = format!("{ASSETS_DIR}/");
    let assets: Vec<(String, Vec<u8>)> = entries
        .iter()
        .filter(|entry| entry.name.starts_with(&prefix))
        .map(|entry| (entry.name.clone(), entry.bytes.clone()))
        .collect();

    Ok(PackageContent {
        manifest,
        manifest_bytes: manifest_bytes.to_vec(),
        module_bytes: module_bytes.to_vec(),
        assets,
        integrity,
        inspection,
        entry_names: entries.iter().map(|entry| entry.name.clone()).collect(),
    })
}

pub fn build_package(input: &PackageInput) -> Result<BuiltPackage, PackageError> {
    let manifest = PluginManifest::parse(&input.manifest_bytes).map_err(|error| {
        PackageError::Manifest(format!("plugin manifest was rejected: {}", error.code()))
    })?;
    let mut entries = vec![
        Entry::new(MANIFEST_FILE, input.manifest_bytes.clone())?,
        Entry::new(MODULE_FILE, input.module_bytes.clone())?,
    ];
    let mut assets: Vec<(String, Vec<u8>)> = input.assets.clone();
    assets.sort_by(|left, right| left.0.cmp(&right.0));
    for (name, bytes) in assets {
        validate_entry_name(&name)?;
        if name == PACKAGE_INTEGRITY_FILE || name == MANIFEST_FILE || name == MODULE_FILE {
            return Err(PackageError::Entry(format!(
                "`{name}` conflicts with a required package entry"
            )));
        }
        entries.push(Entry::new(&name, bytes)?);
    }
    let files = integrity::package_files(&entries)?;
    let integrity = PackageIntegrity::new(
        manifest.id.clone(),
        manifest.version.clone(),
        files,
    )
    .map_err(|error| {
        PackageError::Integrity(format!(
            "package integrity metadata could not be built: {}",
            error.code()
        ))
    })?;
    integrity::build(&entries, &integrity)?;
    entries.push(integrity::integrity_entry(&integrity)?);
    let _ = STORE_METHOD;
    Ok(BuiltPackage { entries, integrity })
}

fn entry<'a>(entries: &'a [Entry], name: &str) -> Option<&'a [u8]> {
    entries
        .iter()
        .find(|entry| entry.name == name)
        .map(|entry| entry.bytes.as_slice())
}
