// Nexus Realm
// File: outcome.rs
// Purpose: Rendered CLI results

use crate::commands::{BuildReport, NewReport, PackReport, ValidationReport};

#[derive(Debug)]
pub enum Outcome {
    Help(String),
    Version(String),
    New(NewReport),
    Build(BuildReport),
    Validate(ValidationReport),
    Pack(PackReport),
}

impl Outcome {
    pub fn render(&self) -> String {
        match self {
            Self::Help(text) => text.clone(),
            Self::Version(text) => format!("{text}\n"),
            Self::New(report) => {
                let mut out = format!(
                    "Created plugin project `{}`\n  plugin id: {}\n  crate: {}\n  sdk: {}\n",
                    report.project_dir.display(),
                    report.plugin_id,
                    report.crate_name,
                    report.sdk_dependency
                );
                for file in &report.files {
                    out.push_str(&format!("  + {file}\n"));
                }
                out.push_str("\nNext: cd into the project, then run `nexus-plugin build`.\n");
                out
            }
            Self::Build(report) => {
                let imports = if report.imports.is_empty() {
                    "none".to_string()
                } else {
                    report.imports.join(", ")
                };
                format!(
                    "Built `{}` for {}\n  artifact: {}\n  staged: {}\n  module: {} bytes\n  exports: {}\n  imports: {}\n",
                    report.crate_name,
                    report.target,
                    report.artifact.display(),
                    report.staged.display(),
                    report.bytes,
                    report.exports.join(", "),
                    imports
                )
            }
            Self::Validate(report) => {
                let mut out = format!(
                    "STRUCTURAL VALIDATION: PASSED ({})\n  source: {}\n  plugin id: {}\n  plugin version: {}\n  plugin API: {}\n  entry: {}\n  permissions: {}\n  subscriptions: {}\n  module: {} bytes\n  exported ABI: {}\n  imported modules: {}\n",
                    report.kind,
                    report.source,
                    report.plugin_id,
                    report.plugin_version,
                    report.api_version,
                    report.entry,
                    if report.permissions.is_empty() { "none".to_string() } else { report.permissions.join(", ") },
                    if report.subscriptions.is_empty() { "none".to_string() } else { report.subscriptions.join(", ") },
                    report.module_bytes,
                    report.exports.join(", "),
                    if report.imports.is_empty() { "none".to_string() } else { report.imports.join(", ") }
                );
                if !report.package_entries.is_empty() {
                    out.push_str(&format!(
                        "  package entries: {}\n  integrity metadata: {}\n",
                        report.package_entries.join(", "),
                        if report.integrity_verified { "verified" } else { "not present" }
                    ));
                }
                for check in &report.checks {
                    out.push_str(&format!("  - {check}\n"));
                }
                out.push_str(
                    "\nStructural validation only: Nexus Realm runtime security, sandboxing, and\npublisher trust are NOT evaluated here, and a structurally valid plugin is not TRUSTED.\n",
                );
                out
            }
            Self::Pack(report) => format!(
                "Packed {} entries into {}\n  size: {} bytes\n  authenticity: UNSIGNED / UNVERIFIED\n  package-integrity.json:\n{}\n",
                report.entries.len(),
                report.output.display(),
                report.bytes,
                report.integrity
            ),
        }
    }
}
