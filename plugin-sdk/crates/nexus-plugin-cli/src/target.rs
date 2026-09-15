// Nexus Realm
// File: target.rs
// Purpose: Official WebAssembly target selection and toolchain detection

use crate::error::CliError;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::Command;

pub const OFFICIAL_WASM_TARGET: &str = "wasm32-wasip1";

pub struct TargetStatus {
    pub target: String,
    pub std_dir: PathBuf,
    pub installed: bool,
}

pub fn cargo_program() -> OsString {
    std::env::var_os("CARGO").unwrap_or_else(|| OsString::from("cargo"))
}

pub fn rustc_program() -> OsString {
    std::env::var_os("RUSTC").unwrap_or_else(|| OsString::from("rustc"))
}

pub fn check_target(target: &str) -> Result<TargetStatus, CliError> {
    let output = Command::new(rustc_program())
        .arg("--print")
        .arg("target-libdir")
        .arg("--target")
        .arg(target)
        .output()
        .map_err(|error| {
            CliError::Toolchain(format!(
                "could not run rustc to resolve the `{target}` standard library: {error}"
            ))
        })?;
    if !output.status.success() {
        return Err(CliError::Toolchain(format!(
            "rustc does not recognize the target `{target}`; the official Nexus Realm plugin target is {OFFICIAL_WASM_TARGET}"
        )));
    }
    let std_dir = PathBuf::from(String::from_utf8_lossy(&output.stdout).trim());
    let installed = directory_has_entries(&std_dir);
    Ok(TargetStatus {
        target: target.to_string(),
        std_dir,
        installed,
    })
}

pub fn require_target(target: &str) -> Result<TargetStatus, CliError> {
    let status = check_target(target)?;
    if !status.installed {
        return Err(CliError::Toolchain(format!(
            "the `{target}` standard library is not installed (expected in {}). Install it with `rustup target add {target}` and run the command again.",
            status.std_dir.display()
        )));
    }
    Ok(status)
}

pub fn target_directory(project_dir: &Path) -> PathBuf {
    match std::env::var_os("CARGO_TARGET_DIR") {
        Some(value) => {
            let path = PathBuf::from(value);
            if path.is_absolute() {
                path
            } else {
                project_dir.join(path)
            }
        }
        None => project_dir.join("target"),
    }
}

pub fn wasm_artifact(project_dir: &Path, crate_name: &str, target: &str) -> PathBuf {
    target_directory(project_dir)
        .join(target)
        .join("release")
        .join(format!("{}.wasm", crate_name.replace('-', "_")))
}

fn directory_has_entries(path: &Path) -> bool {
    match std::fs::read_dir(path) {
        Ok(mut entries) => entries.next().is_some(),
        Err(_) => false,
    }
}
