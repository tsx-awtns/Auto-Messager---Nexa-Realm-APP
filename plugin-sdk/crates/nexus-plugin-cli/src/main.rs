// Nexus Realm
// File: main.rs
// Purpose: nexus-plugin developer CLI entry point

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match nexus_plugin_cli::run(&args) {
        Ok(outcome) => print!("{}", outcome.render()),
        Err(error) => {
            eprintln!("nexus-plugin: {error}");
            std::process::exit(2);
        }
    }
}
