use std::process;

use clap::{Parser, Subcommand};
use svc_mgr::env::EnvScope;

#[derive(Parser)]
#[command(
    name = "renv",
    about = "Cross-platform environment variable management CLI"
)]
struct Cli {
    /// Operate on system-level environment variables (requires admin/root)
    #[arg(long, global = true)]
    system: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// List all environment variables
    List,
    /// Get a specific environment variable
    Get {
        /// Variable name
        key: String,
    },
    /// Set an environment variable
    Set {
        /// Variable name
        key: String,
        /// Variable value
        value: String,
    },
    /// Remove an environment variable
    Unset {
        /// Variable name
        key: String,
    },
}

fn main() {
    env_logger::init();
    let cli = Cli::parse();

    if let Err(err) = run(cli) {
        eprintln!("Error: {err}");
        process::exit(1);
    }
}

fn run(cli: Cli) -> svc_mgr::Result<()> {
    let manager = svc_mgr::env::manager();
    let scope = if cli.system {
        EnvScope::System
    } else {
        EnvScope::User
    };

    match cli.command {
        Commands::List => {
            let vars = manager.list(scope)?;
            let mut keys: Vec<_> = vars.keys().collect();
            keys.sort();

            for key in keys {
                if let Some(value) = vars.get(key) {
                    println!("{}={}", key, value);
                }
            }
        }
        Commands::Get { key } => {
            if let Some(value) = manager.get(scope, &key)? {
                println!("{}", value);
            } else {
                eprintln!("Environment variable '{}' not found", key);
                process::exit(1);
            }
        }
        Commands::Set { key, value } => {
            manager.set(scope, &key, &value)?;
            println!("Set {}={}", key, value);

            if scope == EnvScope::System {
                println!("Note: System environment variables may require a logout/restart to take effect");
            } else {
                println!("Note: You may need to restart your shell for changes to take effect");
            }
        }
        Commands::Unset { key } => {
            manager.unset(scope, &key)?;
            println!("Removed {}", key);

            if scope == EnvScope::System {
                println!("Note: System environment variables may require a logout/restart to take effect");
            } else {
                println!("Note: You may need to restart your shell for changes to take effect");
            }
        }
    }

    Ok(())
}
