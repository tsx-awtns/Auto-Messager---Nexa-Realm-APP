// Nexus Realm
// File: template.rs
// Purpose: Official plugin template materialization and SDK dependency resolution

use crate::error::CliError;
use nexus_plugin_api::{PluginId, PluginManifest, MANIFEST_FILE, PLUGIN_API_VERSION};
use std::fs;
use std::path::{Path, PathBuf};

pub const TEMPLATE_ID: &str = "com.example.basic-plugin";
pub const TEMPLATE_NAME: &str = "Basic Plugin";
pub const TEMPLATE_AUTHOR: &str = "Example Developer";
pub const TEMPLATE_DESCRIPTION: &str = "Basic Nexus Realm plugin template";
pub const TEMPLATE_CRATE: &str = "basic-plugin";
pub const SDK_VERSION: &str = "0.1.0";
pub const SDK_DEPENDENCY_PREFIX: &str = "nexus-plugin-sdk = ";

const CARGO_TOML: &str = include_str!("../../../templates/basic-plugin/Cargo.toml");
const PLUGIN_JSON: &str = include_str!("../../../templates/basic-plugin/plugin.json");
const SOURCE: &str = include_str!("../../../templates/basic-plugin/src/lib.rs");
const README: &str = include_str!("../../../templates/basic-plugin/README.md");

pub struct Generation {
    pub crate_name: String,
    pub plugin_id: String,
    pub display_name: String,
    pub author: String,
    pub sdk_dependency: String,
    pub files: Vec<(String, Vec<u8>)>,
}

pub fn sanitize_crate_name(name: &str) -> Result<String, CliError> {
    let mut sanitized = String::new();
    let mut previous_dash = false;
    for character in name.trim().chars() {
        let lowered = character.to_ascii_lowercase();
        if lowered.is_ascii_alphanumeric() {
            sanitized.push(lowered);
            previous_dash = false;
        } else if !previous_dash && !sanitized.is_empty() {
            sanitized.push('-');
            previous_dash = true;
        }
    }
    while sanitized.ends_with('-') {
        sanitized.pop();
    }
    if sanitized.is_empty() || !sanitized.as_bytes()[0].is_ascii_alphabetic() {
        return Err(CliError::Usage(format!(
            "`{name}` cannot become a Rust crate name; use a name that starts with a letter"
        )));
    }
    if nexus_plugin_api::path::is_reserved_device_name(&sanitized) {
        return Err(CliError::Usage(format!(
            "`{name}` is a reserved device name; choose another project name"
        )));
    }
    Ok(sanitized)
}

pub fn derive_plugin_id(crate_name: &str, explicit: Option<&str>) -> Result<PluginId, CliError> {
    match explicit {
        Some(value) => PluginId::parse(value).map_err(|_| {
            CliError::Usage(format!(
                "`{value}` is not a valid Plugin API V1 plugin id; it must be a reverse-domain id with at least three lowercase labels"
            ))
        }),
        None => PluginId::parse(&format!("com.example.{crate_name}")).map_err(|_| {
            CliError::Usage(format!(
                "`{crate_name}` cannot be used as a plugin id label; pass --id <reverse-domain-id>"
            ))
        }),
    }
}

pub fn display_name(crate_name: &str) -> String {
    let mut out = String::new();
    for part in crate_name.split('-') {
        if part.is_empty() {
            continue;
        }
        if !out.is_empty() {
            out.push(' ');
        }
        let mut characters = part.chars();
        if let Some(first) = characters.next() {
            out.extend(first.to_uppercase());
            out.push_str(characters.as_str());
        }
    }
    if out.is_empty() {
        crate_name.to_string()
    } else {
        out
    }
}

pub fn generate(
    crate_name: &str,
    explicit_id: Option<&str>,
    explicit_author: Option<&str>,
    explicit_sdk: Option<&Path>,
    project_dir: &Path,
) -> Result<Generation, CliError> {
    let plugin_id = derive_plugin_id(crate_name, explicit_id)?;
    let display = display_name(crate_name);
    let author = explicit_author
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| TEMPLATE_AUTHOR.to_string());
    let description = format!("{display} plugin for Nexus Realm");
    let sdk_dependency = resolve_sdk_dependency(project_dir, explicit_sdk)?;

    let manifest = PluginManifest {
        schema_version: nexus_plugin_api::MANIFEST_SCHEMA_VERSION,
        id: plugin_id.clone(),
        name: display.clone(),
        version: "1.0.0".to_string(),
        author: author.clone(),
        description: description.clone(),
        api_version: PLUGIN_API_VERSION.to_string(),
        entry: nexus_plugin_api::MODULE_FILE.to_string(),
        permissions: vec![
            nexus_plugin_api::Permission::EventsSubscribe,
            nexus_plugin_api::Permission::LogsWrite,
        ],
        subscriptions: vec![nexus_plugin_api::EventKind::AppReady],
    };
    manifest.validate().map_err(|error| {
        CliError::Project(format!("generated manifest was rejected: {}", error.code()))
    })?;

    require_literal(PLUGIN_JSON, TEMPLATE_ID)?;
    require_literal(CARGO_TOML, &format!("name = \"{TEMPLATE_CRATE}\""))?;
    require_literal(SOURCE, TEMPLATE_ID)?;
    require_literal(SOURCE, TEMPLATE_NAME)?;
    require_literal(SOURCE, TEMPLATE_AUTHOR)?;
    require_literal(SOURCE, TEMPLATE_DESCRIPTION)?;
    require_literal(CARGO_TOML, SDK_DEPENDENCY_PREFIX)?;

    let cargo_toml = replace_dependency(
        &CARGO_TOML.replace(&format!("name = \"{TEMPLATE_CRATE}\""), &format!("name = \"{crate_name}\"")),
        &sdk_dependency,
    )?;
    let source = SOURCE
        .replace(TEMPLATE_ID, plugin_id.as_str())
        .replace(TEMPLATE_NAME, &display)
        .replace(TEMPLATE_AUTHOR, &author)
        .replace(TEMPLATE_DESCRIPTION, &description);
    let manifest_bytes = serde_json::to_vec_pretty(&manifest)
        .map_err(|_| CliError::Project("manifest serialization failed".to_string()))?;
    let manifest_bytes = [manifest_bytes, b"\n".to_vec()].concat();
    serde_json::from_slice::<PluginManifest>(&manifest_bytes).map_err(|error| {
        CliError::Project(format!("generated manifest is not parseable: {error}"))
    })?;

    let files = vec![
        ("Cargo.toml".to_string(), cargo_toml.into_bytes()),
        (MANIFEST_FILE.to_string(), manifest_bytes),
        ("src/lib.rs".to_string(), source.into_bytes()),
        ("README.md".to_string(), README.as_bytes().to_vec()),
        (
            ".gitignore".to_string(),
            b"target/\ndist/\nplugin.wasm\nCargo.lock\n".to_vec(),
        ),
    ];
    Ok(Generation {
        crate_name: crate_name.to_string(),
        plugin_id: plugin_id.as_str().to_string(),
        display_name: display,
        author,
        sdk_dependency,
        files,
    })
}

fn require_literal(text: &str, literal: &str) -> Result<(), CliError> {
    if text.contains(literal) {
        Ok(())
    } else {
        Err(CliError::Project(format!(
            "the official template no longer contains `{literal}`; template and generator are out of sync"
        )))
    }
}

fn replace_dependency(cargo_toml: &str, dependency: &str) -> Result<String, CliError> {
    let mut out = String::with_capacity(cargo_toml.len() + dependency.len());
    let mut replaced = false;
    for line in cargo_toml.lines() {
        if line.trim_start().starts_with(SDK_DEPENDENCY_PREFIX) {
            out.push_str(&format!(
                "{}nexus-plugin-sdk = {dependency}",
                &line[..line.len() - line.trim_start().len()]
            ));
            replaced = true;
        } else {
            out.push_str(line);
        }
        out.push('\n');
    }
    if !replaced {
        return Err(CliError::Project(
            "the official template does not declare the plugin SDK".to_string(),
        ));
    }
    Ok(out)
}

pub fn resolve_sdk_dependency(
    project_dir: &Path,
    explicit: Option<&Path>,
) -> Result<String, CliError> {
    if let Some(path) = explicit {
        let resolved = if path.is_absolute() {
            path.to_path_buf()
        } else {
            std::env::current_dir()
                .map_err(|error| CliError::Io(error.to_string()))?
                .join(path)
        };
        if !resolved.join("Cargo.toml").is_file() {
            return Err(CliError::Usage(format!(
                "`{}` does not contain a nexus-plugin-sdk crate",
                path.display()
            )));
        }
        let relative = relative_path(project_dir, &resolved)
            .unwrap_or_else(|| resolved.to_string_lossy().to_string());
        return Ok(format!(
            "{{ path = \"{}\", version = \"{SDK_VERSION}\" }}",
            relative.replace('\\', "/")
        ));
    }
    if let Some(sdk_dir) = find_workspace_sdk(project_dir) {
        if let Some(relative) = relative_path(project_dir, &sdk_dir) {
            return Ok(format!(
                "{{ path = \"{}\", version = \"{SDK_VERSION}\" }}",
                relative.replace('\\', "/")
            ));
        }
    }
    if let Some(sdk_dir) = current_directory_sdk() {
        if let Some(relative) = relative_path(project_dir, &sdk_dir) {
            return Ok(format!(
                "{{ path = \"{}\", version = \"{SDK_VERSION}\" }}",
                relative.replace('\\', "/")
            ));
        }
    }
    if let Some(sdk_dir) = executable_sdk() {
        if let Some(relative) = relative_path(project_dir, &sdk_dir) {
            return Ok(format!(
                "{{ path = \"{}\", version = \"{SDK_VERSION}\" }}",
                relative.replace('\\', "/")
            ));
        }
    }
    Ok(format!("{{ version = \"{SDK_VERSION}\" }}"))
}

fn find_workspace_sdk(start: &Path) -> Option<PathBuf> {
    let mut current = Some(start);
    while let Some(dir) = current {
        let candidate = dir
            .join("plugin-sdk")
            .join("crates")
            .join("nexus-plugin-sdk");
        if candidate.join("Cargo.toml").is_file() {
            return Some(candidate);
        }
        let nested = dir.join("crates").join("nexus-plugin-sdk");
        if nested.join("Cargo.toml").is_file() {
            return Some(nested);
        }
        current = dir.parent();
    }
    None
}

fn current_directory_sdk() -> Option<PathBuf> {
    let cwd = std::env::current_dir().ok()?;
    find_workspace_sdk(&cwd)
}

fn executable_sdk() -> Option<PathBuf> {
    let exe = std::env::current_exe().ok()?;
    let dir = exe.parent()?;
    find_workspace_sdk(dir)
}

pub fn require_sdk_dependency(project_dir: &Path) -> Result<(), CliError> {
    let text = fs::read_to_string(project_dir.join("Cargo.toml"))
        .map_err(|_| CliError::Project("Cargo.toml is missing".to_string()))?;
    if text
        .lines()
        .any(|line| line.trim_start().starts_with(SDK_DEPENDENCY_PREFIX))
    {
        Ok(())
    } else {
        Err(CliError::Project(
            "Cargo.toml does not depend on nexus-plugin-sdk".to_string(),
        ))
    }
}

pub fn relative_path(from: &Path, to: &Path) -> Option<String> {
    let from = from.canonicalize().ok()?;
    let to = to.canonicalize().ok()?;
    let from_parts: Vec<_> = from.components().collect();
    let to_parts: Vec<_> = to.components().collect();
    if from_parts.first() != to_parts.first() {
        return None;
    }
    let mut common = 0usize;
    while common < from_parts.len().min(to_parts.len())
        && from_parts[common] == to_parts[common]
    {
        common += 1;
    }
    let mut segments: Vec<String> = Vec::new();
    for _ in common..from_parts.len() {
        segments.push("..".to_string());
    }
    for part in &to_parts[common..] {
        segments.push(part.as_os_str().to_string_lossy().to_string());
    }
    if segments.is_empty() {
        segments.push(".".to_string());
    }
    Some(segments.join("/"))
}
