// Nexus Realm
// File: lib.rs
// Purpose: Nexus Realm plugin developer CLI

pub mod cli;
pub mod commands;
pub mod error;
pub mod integrity;
pub mod outcome;
pub mod package;
pub mod target;
pub mod template;
pub mod wasm;

pub use cli::{Command, BuildOptions, NewOptions, PackOptions, ValidateOptions};
pub use error::CliError;
pub use outcome::Outcome;

pub fn run(args: &[String]) -> Result<Outcome, CliError> {
    match cli::parse(args)? {
        Command::Help(topic) => Ok(Outcome::Help(cli::usage(topic.as_deref())?)),
        Command::Version => Ok(Outcome::Version(format!(
            "nexus-plugin {} (plugin API {})",
            env!("CARGO_PKG_VERSION"),
            nexus_plugin_api::PLUGIN_API_VERSION
        ))),
        Command::New(options) => Ok(Outcome::New(commands::new_project(&options)?)),
        Command::Build(options) => Ok(Outcome::Build(commands::build_project(&options)?)),
        Command::Validate(options) => Ok(Outcome::Validate(commands::validate(&options)?)),
        Command::Pack(options) => Ok(Outcome::Pack(commands::pack_project(&options)?)),
    }
}
