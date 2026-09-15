// Nexus Realm
// File: cli.rs
// Purpose: Command surface and argument parsing

use crate::error::CliError;
use crate::target::OFFICIAL_WASM_TARGET;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    Help(Option<String>),
    Version,
    New(NewOptions),
    Build(BuildOptions),
    Validate(ValidateOptions),
    Pack(PackOptions),
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct NewOptions {
    pub name: String,
    pub id: Option<String>,
    pub author: Option<String>,
    pub directory: Option<PathBuf>,
    pub sdk: Option<PathBuf>,
    pub force: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildOptions {
    pub path: PathBuf,
    pub target: String,
    pub offline: bool,
}

impl Default for BuildOptions {
    fn default() -> Self {
        Self {
            path: PathBuf::from("."),
            target: OFFICIAL_WASM_TARGET.to_string(),
            offline: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidateOptions {
    pub path: PathBuf,
    pub package: Option<PathBuf>,
}

impl Default for ValidateOptions {
    fn default() -> Self {
        Self {
            path: PathBuf::from("."),
            package: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackOptions {
    pub path: PathBuf,
    pub out: Option<PathBuf>,
    pub name: Option<String>,
}

impl Default for PackOptions {
    fn default() -> Self {
        Self {
            path: PathBuf::from("."),
            out: None,
            name: None,
        }
    }
}

struct Flags {
    values: Vec<(String, String)>,
    switches: Vec<String>,
    positionals: Vec<String>,
}

impl Flags {
    fn parse(
        args: &[String],
        value_flags: &[&str],
        switch_flags: &[&str],
    ) -> Result<Self, CliError> {
        let mut flags = Self {
            values: Vec::new(),
            switches: Vec::new(),
            positionals: Vec::new(),
        };
        let mut index = 0;
        while index < args.len() {
            let item = args[index].as_str();
            if let Some(rest) = item.strip_prefix("--") {
                let (name, inline) = match rest.split_once('=') {
                    Some((name, value)) => (name, Some(value.to_string())),
                    None => (rest, None),
                };
                if value_flags.contains(&name) {
                    let value = match inline {
                        Some(value) => value,
                        None => {
                            index += 1;
                            args.get(index)
                                .cloned()
                                .ok_or_else(|| CliError::Usage(format!("`--{name}` requires a value")))?
                        }
                    };
                    if value.is_empty() {
                        return Err(CliError::Usage(format!("`--{name}` requires a value")));
                    }
                    flags.values.push((name.to_string(), value));
                } else if switch_flags.contains(&name) {
                    if inline.is_some() {
                        return Err(CliError::Usage(format!("`--{name}` does not take a value")));
                    }
                    flags.switches.push(name.to_string());
                } else {
                    return Err(CliError::Usage(format!("unknown option `--{name}`")));
                }
            } else if item.starts_with('-') && item.len() > 1 {
                return Err(CliError::Usage(format!("unknown option `{item}`")));
            } else {
                flags.positionals.push(item.to_string());
            }
            index += 1;
        }
        Ok(flags)
    }

    fn value(&self, name: &str) -> Option<&str> {
        self.values
            .iter()
            .find(|(key, _)| key == name)
            .map(|(_, value)| value.as_str())
    }

    fn switch(&self, name: &str) -> bool {
        self.switches.iter().any(|key| key == name)
    }

    fn take_value(&self, name: &str) -> Option<String> {
        self.value(name).map(|value| value.to_string())
    }
}

pub fn parse(args: &[String]) -> Result<Command, CliError> {
    let Some((head, rest)) = args.split_first() else {
        return Ok(Command::Help(None));
    };
    match head.as_str() {
        "help" => {
            if let Some(topic) = rest.first() {
                if !["new", "build", "validate", "pack"].contains(&topic.as_str()) {
                    return Err(CliError::Usage(format!("unknown help topic `{topic}`")));
                }
            }
            Ok(Command::Help(rest.first().cloned()))
        }
        "--help" | "-h" => Ok(Command::Help(None)),
        "--version" | "-V" | "version" => Ok(Command::Version),
        "new" => {
            let flags = Flags::parse(rest, &["id", "author", "dir", "sdk"], &["force"])?;
            if flags.positionals.len() != 1 {
                return Err(CliError::Usage(
                    "`new` requires exactly one project name: nexus-plugin new <name>".to_string(),
                ));
            }
            Ok(Command::New(NewOptions {
                name: flags.positionals[0].clone(),
                id: flags.take_value("id"),
                author: flags.take_value("author"),
                directory: flags.value("dir").map(PathBuf::from),
                sdk: flags.value("sdk").map(PathBuf::from),
                force: flags.switch("force"),
            }))
        }
        "build" => {
            let flags = Flags::parse(rest, &["path", "target"], &["offline"])?;
            if !flags.positionals.is_empty() {
                return Err(CliError::Usage(
                    "`build` takes no positional arguments; use `--path <project>`".to_string(),
                ));
            }
            Ok(Command::Build(BuildOptions {
                path: flags
                    .value("path")
                    .map(PathBuf::from)
                    .unwrap_or_else(|| PathBuf::from(".")),
                target: flags
                    .take_value("target")
                    .unwrap_or_else(|| OFFICIAL_WASM_TARGET.to_string()),
                offline: flags.switch("offline"),
            }))
        }
        "validate" => {
            let flags = Flags::parse(rest, &["path", "package"], &[])?;
            if !flags.positionals.is_empty() {
                return Err(CliError::Usage(
                    "`validate` takes no positional arguments; use `--path <project>` or `--package <file>`"
                        .to_string(),
                ));
            }
            let options = ValidateOptions {
                path: flags
                    .value("path")
                    .map(PathBuf::from)
                    .unwrap_or_else(|| PathBuf::from(".")),
                package: flags.value("package").map(PathBuf::from),
            };
            if options.package.is_some() && flags.value("path").is_some() {
                return Err(CliError::Usage(
                    "`validate` accepts either `--path` or `--package`, not both".to_string(),
                ));
            }
            Ok(Command::Validate(options))
        }
        "pack" => {
            let flags = Flags::parse(rest, &["path", "out", "name"], &[])?;
            if !flags.positionals.is_empty() {
                return Err(CliError::Usage(
                    "`pack` takes no positional arguments; use `--path <project>`".to_string(),
                ));
            }
            if let Some(name) = flags.value("name") {
                if name.contains(['/', '\\']) || nexus_plugin_api::path::validate_relative(name).is_err()
                {
                    return Err(CliError::Usage(
                        "`--name` must be a plain file name without path separators".to_string(),
                    ));
                }
            }
            Ok(Command::Pack(PackOptions {
                path: flags
                    .value("path")
                    .map(PathBuf::from)
                    .unwrap_or_else(|| PathBuf::from(".")),
                out: flags.value("out").map(PathBuf::from),
                name: flags.take_value("name"),
            }))
        }
        other => Err(CliError::Usage(format!(
            "unknown command `{other}`; run `nexus-plugin help`"
        ))),
    }
}

pub fn usage(topic: Option<&str>) -> Result<String, CliError> {
    let general = format!(
        "nexus-plugin {}

USAGE
    nexus-plugin <command> [options]

COMMANDS
    new <name>      Create a Rust plugin project from the official template
    build           Build the project for the official WebAssembly target
    validate        Validate a plugin project or .nexusplugin package structurally
    pack            Build a .nexusplugin package from a built project
    help [command]  Show this help or command-specific help

GLOBAL
    --version, -V   Print the CLI and Plugin API version
    --help, -h      Print this help

The official WebAssembly target is {OFFICIAL_WASM_TARGET}. Nexus Realm validates a package,
installs it through the Plugin Manager after explicit permission consent, and runs it inside
a Wasmtime sandbox with bounded memory, fuel, and time. Structural validation and package
integrity are not publisher authenticity: packages this CLI produces are UNSIGNED and stay
UNVERIFIED until publisher signing exists.
",
        env!("CARGO_PKG_VERSION")
    );
    let specific = match topic {
        None => None,
        Some("new") => Some(
            "nexus-plugin new <name> [--id <plugin-id>] [--author <name>] [--dir <path>] [--sdk <path>] [--force]\n\
             \n\
             Creates <name>/ with Cargo.toml, plugin.json, src/lib.rs, README.md, and assets/.\n\
             The plugin id defaults to com.example.<name> and must satisfy Plugin API Manifest V1.\n"
                .to_string(),
        ),
        Some("build") => Some(format!(
            "nexus-plugin build [--path <project>] [--target <triple>] [--offline]\n\
             \n\
             Runs `cargo build --release --target {OFFICIAL_WASM_TARGET}` inside the project and stages the\n\
             resulting module as <project>/plugin.wasm. The target must already be installed\n\
             (`rustup target add {OFFICIAL_WASM_TARGET}`); this command never installs toolchains.\n"
        )),
        Some("validate") => Some(
            "nexus-plugin validate [--path <project>] [--package <file.nexusplugin>]\n\
             \n\
             Structural validation: container, manifest V1, limits, path safety, module header,\n\
             and package integrity. Nexus Realm performs runtime admission, sandboxing, and\n\
             permission enforcement separately when the plugin is installed and enabled, and\n\
             publisher authenticity is never evaluated, so a valid plugin stays UNVERIFIED.\n"
                .to_string(),
        ),
        Some("pack") => Some(
            "nexus-plugin pack [--path <project>] [--out <dir>] [--name <file>]\n\
             \n\
             Packages plugin.json, plugin.wasm, assets/, and package-integrity.json into\n\
             <out>/<plugin-short-name>-<version>.nexusplugin. Output is unsigned and unverified.\n"
                .to_string(),
        ),
        Some(other) => {
            return Err(CliError::Usage(format!("unknown help topic `{other}`")));
        }
    };
    Ok(match specific {
        Some(text) => text,
        None => general,
    })
}
