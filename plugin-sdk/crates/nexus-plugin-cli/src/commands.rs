// Nexus Realm
// File: commands.rs
// Purpose: Project generation, building, structural validation, and packing

use crate::cli::{BuildOptions, NewOptions, PackOptions, ValidateOptions};
use crate::error::CliError;
use crate::package;
use crate::target;
use crate::template;
use crate::wasm;
use nexus_plugin_api::{
    forbidden_suffix_of, path, PluginManifest, ASSETS_DIR, MANIFEST_FILE, MAX_MANIFEST_BYTES,
    MAX_MODULE_BYTES, MAX_PLUGIN_BYTES, MAX_PLUGIN_DEPTH, MAX_PLUGIN_FILES, MODULE_FILE,
};
use nexus_plugin_package::{build_package, read_package, PackageInput};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug)]
pub struct NewReport {
    pub project_dir: PathBuf,
    pub plugin_id: String,
    pub crate_name: String,
    pub display_name: String,
    pub author: String,
    pub sdk_dependency: String,
    pub files: Vec<String>,
}

#[derive(Debug)]
pub struct BuildReport {
    pub crate_name: String,
    pub target: String,
    pub artifact: PathBuf,
    pub staged: PathBuf,
    pub bytes: u64,
    pub exports: Vec<String>,
    pub imports: Vec<String>,
}

#[derive(Debug)]
pub struct ValidationReport {
    pub kind: &'static str,
    pub source: String,
    pub plugin_id: String,
    pub plugin_version: String,
    pub api_version: String,
    pub entry: String,
    pub permissions: Vec<String>,
    pub subscriptions: Vec<String>,
    pub module_bytes: u64,
    pub exports: Vec<String>,
    pub imports: Vec<String>,
    pub checks: Vec<String>,
    pub package_entries: Vec<String>,
    pub integrity_verified: bool,
}

#[derive(Debug)]
pub struct PackReport {
    pub output: PathBuf,
    pub entries: Vec<String>,
    pub integrity: String,
    pub bytes: u64,
}

pub fn new_project(options: &NewOptions) -> Result<NewReport, CliError> {
    let crate_name = template::sanitize_crate_name(&options.name)?;
    template::derive_plugin_id(&crate_name, options.id.as_deref())?;
    let parent = options
        .directory
        .clone()
        .unwrap_or_else(|| PathBuf::from("."));
    let project_dir = parent.join(&crate_name);
    if project_dir.exists() && !options.force {
        let populated = fs::read_dir(&project_dir)
            .map(|mut entries| entries.next().is_some())
            .unwrap_or(true);
        if populated {
            return Err(CliError::Conflict(format!(
                "`{}` already exists; pass --force to overwrite the generated files",
                project_dir.display()
            )));
        }
    }
    fs::create_dir_all(&project_dir)?;
    fs::create_dir_all(project_dir.join(ASSETS_DIR))?;
    let generation = template::generate(
        &crate_name,
        options.id.as_deref(),
        options.author.as_deref(),
        options.sdk.as_deref(),
        &project_dir,
    )?;
    let mut files = Vec::new();
    for (relative, contents) in &generation.files {
        let destination = project_dir.join(relative);
        if let Some(parent) = destination.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&destination, contents)?;
        files.push(relative.clone());
    }
    files.push(format!("{ASSETS_DIR}/"));
    Ok(NewReport {
        project_dir,
        plugin_id: generation.plugin_id,
        crate_name: generation.crate_name,
        display_name: generation.display_name,
        author: generation.author,
        sdk_dependency: generation.sdk_dependency,
        files,
    })
}

pub fn build_project(options: &BuildOptions) -> Result<BuildReport, CliError> {
    let project_dir = canonical_directory(&options.path)?;
    let crate_name = read_crate_name(&project_dir)?;
    let manifest_bytes = read_bounded(&project_dir.join(MANIFEST_FILE), MAX_MANIFEST_BYTES, MANIFEST_FILE)?;
    PluginManifest::parse(&manifest_bytes)?;
    template::require_sdk_dependency(&project_dir)?;
    target::require_target(&options.target)?;

    let mut command = Command::new(target::cargo_program());
    command
        .current_dir(&project_dir)
        .arg("build")
        .arg("--release")
        .arg("--target")
        .arg(&options.target);
    if options.offline {
        command.arg("--offline");
    }
    let status = command
        .status()
        .map_err(|error| CliError::Command(format!("could not start cargo: {error}")))?;
    if !status.success() {
        return Err(CliError::Command(format!(
            "cargo build failed with {status}; the module was not staged"
        )));
    }

    let artifact = target::wasm_artifact(&project_dir, &crate_name, &options.target);
    let bytes = fs::read(&artifact).map_err(|_| {
        CliError::Command(format!(
            "cargo reported success but `{}` is missing",
            artifact.display()
        ))
    })?;
    if bytes.len() as u64 > MAX_MODULE_BYTES {
        return Err(CliError::Wasm("module exceeds the module size limit".to_string()));
    }
    let inspection = wasm::inspect(&bytes)?;
    let missing = inspection.missing_abi_exports();
    if !missing.is_empty() {
        return Err(CliError::Wasm(format!(
            "the built module does not export the Plugin API v1 surface ({}); use nexus_plugin_sdk::export_plugin!",
            missing.join(", ")
        )));
    }
    let staged = project_dir.join(MODULE_FILE);
    write_atomic(&staged, &bytes)?;
    Ok(BuildReport {
        crate_name,
        target: options.target.clone(),
        artifact,
        staged,
        bytes: bytes.len() as u64,
        exports: inspection.exports,
        imports: inspection.imports,
    })
}

pub fn validate(options: &ValidateOptions) -> Result<ValidationReport, CliError> {
    match &options.package {
        Some(file) => validate_package(file),
        None => validate_project(&options.path),
    }
}

pub fn validate_project(path: &Path) -> Result<ValidationReport, CliError> {
    let project_dir = canonical_directory(path)?;
    let manifest_bytes = read_bounded(&project_dir.join(MANIFEST_FILE), MAX_MANIFEST_BYTES, MANIFEST_FILE)?;
    let module_bytes = read_bounded(&project_dir.join(MODULE_FILE), MAX_MODULE_BYTES, MODULE_FILE)?;
    let mut checks = vec![
        "project layout: plugin.json and plugin.wasm are regular files".to_string(),
        "no links found in package content".to_string(),
    ];
    let assets_dir = project_dir.join(ASSETS_DIR);
    let assets = match fs::symlink_metadata(&assets_dir) {
        Ok(metadata) => {
            if is_link(&metadata) || !metadata.is_dir() {
                return Err(CliError::Project(format!(
                    "`{ASSETS_DIR}` must be a regular directory"
                )));
            }
            collect_assets(&assets_dir)?
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Vec::new(),
        Err(error) => return Err(CliError::Io(error.to_string())),
    };
    let asset_bytes: u64 = assets.iter().map(|(_, bytes)| bytes.len() as u64).sum();
    checks.push(format!(
        "assets: {} file(s), {asset_bytes} byte(s)",
        assets.len()
    ));
    let (manifest, inspection) = inspect_content(&manifest_bytes, &module_bytes)?;
    checks.extend(content_checks(&manifest, &inspection, &module_bytes));
    Ok(ValidationReport {
        kind: "project",
        source: project_dir.display().to_string(),
        plugin_id: manifest.id.as_str().to_string(),
        plugin_version: manifest.version.clone(),
        api_version: manifest.api_version.clone(),
        entry: manifest.entry.clone(),
        permissions: manifest
            .permissions
            .iter()
            .map(|permission| permission.as_str().to_string())
            .collect(),
        subscriptions: manifest
            .subscriptions
            .iter()
            .map(|kind| kind.as_str().to_string())
            .collect(),
        module_bytes: module_bytes.len() as u64,
        exports: inspection.exports,
        imports: inspection.imports,
        checks,
        package_entries: Vec::new(),
        integrity_verified: false,
    })
}

pub fn validate_package(file: &Path) -> Result<ValidationReport, CliError> {
    let limit = MAX_PLUGIN_BYTES + 1024 * 1024;
    let bytes = read_bounded(file, limit, "package")?;
    let content = read_package(&bytes)?;
    let manifest = content.manifest;
    let inspection = content.inspection;
    let module_bytes = content.module_bytes;
    let mut checks = vec![
        "package container: safe entry names, no duplicates, stored entries only".to_string(),
        "package integrity: SHA-256 and sizes verified for every entry".to_string(),
        "package integrity: authenticity is unsigned".to_string(),
    ];
    checks.extend(content_checks(&manifest, &inspection, &module_bytes));
    Ok(ValidationReport {
        kind: "package",
        source: file.display().to_string(),
        plugin_id: manifest.id.as_str().to_string(),
        plugin_version: manifest.version.clone(),
        api_version: manifest.api_version.clone(),
        entry: manifest.entry.clone(),
        permissions: manifest
            .permissions
            .iter()
            .map(|permission| permission.as_str().to_string())
            .collect(),
        subscriptions: manifest
            .subscriptions
            .iter()
            .map(|kind| kind.as_str().to_string())
            .collect(),
        module_bytes: module_bytes.len() as u64,
        exports: inspection.exports,
        imports: inspection.imports,
        checks,
        package_entries: content.entry_names,
        integrity_verified: true,
    })
}

pub fn pack_project(options: &PackOptions) -> Result<PackReport, CliError> {
    let project_dir = canonical_directory(&options.path)?;
    let manifest_bytes = read_bounded(&project_dir.join(MANIFEST_FILE), MAX_MANIFEST_BYTES, MANIFEST_FILE)?;
    let module_bytes = read_bounded(&project_dir.join(MODULE_FILE), MAX_MODULE_BYTES, MODULE_FILE)?;
    let (manifest, _) = inspect_content(&manifest_bytes, &module_bytes)?;
    let mut assets: Vec<(String, Vec<u8>)> = Vec::new();
    let assets_dir = project_dir.join(ASSETS_DIR);
    if let Ok(metadata) = fs::symlink_metadata(&assets_dir) {
        if is_link(&metadata) || !metadata.is_dir() {
            return Err(CliError::Project(format!(
                "`{ASSETS_DIR}` must be a regular directory"
            )));
        }
        assets = collect_assets(&assets_dir)?;
    }
    let built = build_package(&PackageInput {
        manifest_bytes: manifest_bytes.clone(),
        module_bytes: module_bytes.clone(),
        assets,
    })?;
    let archive = package::write_archive(&built.entries)?;
    let verified = read_package(&archive)?;
    if verified.manifest.id != manifest.id {
        return Err(CliError::Package(
            "packaged plugin identity does not match the project manifest".to_string(),
        ));
    }

    let out_dir = options
        .out
        .clone()
        .unwrap_or_else(|| project_dir.join("dist"));
    fs::create_dir_all(&out_dir)?;
    let file_name = options.name.clone().unwrap_or_else(|| {
        format!(
            "{}-{}.{}",
            manifest.id.short_name(),
            manifest.version,
            package::EXTENSION
        )
    });
    let output = out_dir.join(&file_name);
    write_atomic(&output, &archive)?;
    Ok(PackReport {
        output,
        entries: built.entries.iter().map(|entry| entry.name.clone()).collect(),
        integrity: built.integrity.to_canonical_json(),
        bytes: archive.len() as u64,
    })
}

fn inspect_content(
    manifest_bytes: &[u8],
    module_bytes: &[u8],
) -> Result<(PluginManifest, wasm::WasmInspection), CliError> {
    let manifest = PluginManifest::parse(manifest_bytes)?;
    if module_bytes.len() as u64 > MAX_MODULE_BYTES {
        return Err(CliError::Wasm(
            "module exceeds the module size limit".to_string(),
        ));
    }
    let inspection = wasm::inspect(module_bytes)?;
    let missing = inspection.missing_abi_exports();
    if !missing.is_empty() {
        return Err(CliError::Wasm(format!(
            "module does not export the Plugin API v1 surface ({})",
            missing.join(", ")
        )));
    }
    Ok((manifest, inspection))
}

fn content_checks(
    manifest: &PluginManifest,
    inspection: &wasm::WasmInspection,
    module_bytes: &[u8],
) -> Vec<String> {
    let mut checks = vec![
        format!(
            "manifest: schemaVersion {}, apiVersion {}, entry {}",
            manifest.schema_version, manifest.api_version, manifest.entry
        ),
        format!("plugin id: {} is a valid reverse-domain identifier", manifest.id),
        format!(
            "permissions: {} grantable identifier(s), no protected capability requested",
            manifest.permissions.len()
        ),
        format!(
            "subscriptions: {} identifier(s), events.subscribe requirement satisfied",
            manifest.subscriptions.len()
        ),
        format!(
            "module: WebAssembly version {}, {} byte(s)",
            inspection.version,
            module_bytes.len()
        ),
        "module: Plugin API v1 exports present".to_string(),
        format!(
            "module imports: {}",
            if inspection.imports.is_empty() {
                "none".to_string()
            } else {
                inspection.imports.join(", ")
            }
        ),
        "content: no native executable payload present".to_string(),
    ];
    checks.push(format!(
        "plugin API: {} (validated against the authoritative contract)",
        nexus_plugin_api::PLUGIN_API_VERSION
    ));
    checks
}

fn collect_assets(root: &Path) -> Result<Vec<(String, Vec<u8>)>, CliError> {
    let mut collected: Vec<(String, Vec<u8>)> = Vec::new();
    let mut total = 0u64;
    let mut stack = vec![(root.to_path_buf(), ASSETS_DIR.to_string(), 0usize)];
    while let Some((directory, relative, depth)) = stack.pop() {
        if depth > MAX_PLUGIN_DEPTH {
            return Err(CliError::Project(format!(
                "`{relative}` exceeds the asset nesting limit"
            )));
        }
        let mut entries: Vec<_> = fs::read_dir(&directory)?
            .collect::<Result<Vec<_>, _>>()?;
        entries.sort_by_key(|entry| entry.file_name());
        for entry in entries {
            let name = entry.file_name().to_string_lossy().to_string();
            let child = format!("{relative}/{name}");
            path::validate_relative(&child).map_err(|_| {
                CliError::Project(format!("`{child}` is not a safe plugin path"))
            })?;
            if forbidden_suffix_of(&child).is_some() {
                return Err(CliError::Project(format!(
                    "`{child}` is a forbidden native payload"
                )));
            }
            let metadata = fs::symlink_metadata(entry.path())?;
            if is_link(&metadata) {
                return Err(CliError::Project(format!(
                    "`{child}` is a link; plugins may not contain links"
                )));
            }
            if metadata.is_dir() {
                stack.push((entry.path(), child, depth + 1));
            } else if metadata.is_file() {
                if collected.len() >= MAX_PLUGIN_FILES {
                    return Err(CliError::Project(
                        "the plugin contains too many files".to_string(),
                    ));
                }
                total = total
                    .checked_add(metadata.len())
                    .ok_or_else(|| CliError::Project("plugin size overflowed".to_string()))?;
                if total > MAX_PLUGIN_BYTES {
                    return Err(CliError::Project(
                        "the plugin exceeds the plugin size limit".to_string(),
                    ));
                }
                collected.push((child, fs::read(entry.path())?));
            } else {
                return Err(CliError::Project(format!(
                    "`{child}` is neither a regular file nor a directory"
                )));
            }
        }
    }
    collected.sort_by(|left, right| left.0.cmp(&right.0));
    Ok(collected)
}

fn canonical_directory(path: &Path) -> Result<PathBuf, CliError> {
    let resolved = fs::canonicalize(path).map_err(|_| {
        CliError::Project(format!("`{}` was not found", path.display()))
    })?;
    let metadata = fs::symlink_metadata(&resolved)?;
    if !metadata.is_dir() {
        return Err(CliError::Project(format!(
            "`{}` is not a directory",
            path.display()
        )));
    }
    Ok(resolved)
}

fn read_bounded(path: &Path, limit: u64, label: &str) -> Result<Vec<u8>, CliError> {
    let metadata = fs::symlink_metadata(path).map_err(|_| {
        CliError::Project(format!("`{label}` is missing"))
    })?;
    if is_link(&metadata) {
        return Err(CliError::Project(format!(
            "`{label}` is a link; plugins may not contain links"
        )));
    }
    if !metadata.is_file() {
        return Err(CliError::Project(format!(
            "`{label}` is not a regular file"
        )));
    }
    if metadata.len() > limit {
        return Err(CliError::Project(format!(
            "`{label}` exceeds its size limit"
        )));
    }
    Ok(fs::read(path)?)
}

fn read_crate_name(project_dir: &Path) -> Result<String, CliError> {
    let text = fs::read_to_string(project_dir.join("Cargo.toml")).map_err(|_| {
        CliError::Project("Cargo.toml is missing".to_string())
    })?;
    let mut in_package = false;
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') {
            in_package = trimmed == "[package]";
            continue;
        }
        if !in_package {
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix("name") {
            let rest = rest.trim_start();
            if let Some(value) = rest.strip_prefix('=') {
                let value = value.trim();
                if let Some(value) = value.strip_prefix('"').and_then(|value| value.strip_suffix('"')) {
                    if !value.is_empty() {
                        return Ok(value.to_string());
                    }
                }
            }
        }
    }
    Err(CliError::Project(
        "Cargo.toml does not declare a package name".to_string(),
    ))
}

fn write_atomic(path: &Path, bytes: &[u8]) -> Result<(), CliError> {
    let parent = path.parent().ok_or_else(|| {
        CliError::Io("output path has no parent directory".to_string())
    })?;
    fs::create_dir_all(parent)?;
    let name = path
        .file_name()
        .map(|value| value.to_string_lossy().to_string())
        .unwrap_or_else(|| "output".to_string());
    let temporary = parent.join(format!(".{name}.tmp-{}", std::process::id()));
    fs::write(&temporary, bytes)?;
    if path.exists() {
        fs::remove_file(path)?;
    }
    fs::rename(&temporary, path)?;
    Ok(())
}

fn is_link(metadata: &fs::Metadata) -> bool {
    if metadata.file_type().is_symlink() {
        return true;
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        if metadata.file_attributes() & 0x400 != 0 {
            return true;
        }
    }
    false
}
