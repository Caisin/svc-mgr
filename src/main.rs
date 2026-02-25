use std::process;

use clap::{Parser, Subcommand, ValueEnum};
use svc_mgr::{
    ActionOutput, RestartPolicy, ServiceAction, ServiceBuilder, ServiceLevel, ServiceManager,
    ServiceManagerKind, ServiceStatus, TypedServiceManager,
};

#[derive(Parser)]
#[command(name = "rsvc", about = "Cross-platform service management CLI")]
struct Cli {
    /// Manage user-level services instead of system-level
    #[arg(long, global = true)]
    user: bool,

    /// Specify backend (default: auto-detect)
    #[arg(long, value_enum, global = true)]
    backend: Option<Backend>,

    /// Preview commands without executing
    #[arg(long, global = true)]
    dry_run: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Clone, ValueEnum)]
enum Backend {
    Launchd,
    Systemd,
    Openrc,
    Rcd,
    Sc,
    Winsw,
}

impl From<Backend> for ServiceManagerKind {
    fn from(b: Backend) -> Self {
        match b {
            Backend::Launchd => Self::Launchd,
            Backend::Systemd => Self::Systemd,
            Backend::Openrc => Self::OpenRc,
            Backend::Rcd => Self::Rcd,
            Backend::Sc => Self::Sc,
            Backend::Winsw => Self::WinSw,
        }
    }
}

#[derive(Subcommand)]
enum Commands {
    /// Install a new service
    Install {
        /// Service label (e.g. com.example.myapp)
        label: String,
        /// Path to the program executable
        #[arg(long)]
        program: String,
        /// Arguments to pass to the program
        #[arg(long, num_args = 1..)]
        args: Vec<String>,
        /// Working directory
        #[arg(long)]
        workdir: Option<String>,
        /// Environment variables (KEY=VALUE)
        #[arg(long, num_args = 1..)]
        env: Vec<String>,
        /// Run as this user
        #[arg(long)]
        username: Option<String>,
        /// Service description
        #[arg(long)]
        description: Option<String>,
        /// Start service automatically on boot
        #[arg(long)]
        autostart: bool,
        /// Restart policy: never, always, on-failure, on-success
        #[arg(long, default_value = "on-failure")]
        restart: String,
        /// Delay in seconds before restart
        #[arg(long)]
        restart_delay: Option<u32>,
        /// Maximum retry count (only for on-failure)
        #[arg(long)]
        max_retries: Option<u32>,
    },
    /// Uninstall a service
    Uninstall {
        /// Service label
        label: String,
    },
    /// Start a service
    Start {
        /// Service label
        label: String,
    },
    /// Stop a service
    Stop {
        /// Service label
        label: String,
    },
    /// Restart a service
    Restart {
        /// Service label
        label: String,
    },
    /// Query service status
    Status {
        /// Service label
        label: String,
    },
    /// List installed services
    List,
}

fn parse_restart_policy(
    restart: &str,
    delay: Option<u32>,
    max_retries: Option<u32>,
) -> RestartPolicy {
    match restart {
        "never" => RestartPolicy::Never,
        "always" => RestartPolicy::Always {
            delay_secs: delay,
        },
        "on-failure" => RestartPolicy::OnFailure {
            delay_secs: delay,
            max_retries,
            reset_after_secs: None,
        },
        "on-success" => RestartPolicy::OnSuccess {
            delay_secs: delay,
        },
        other => {
            eprintln!("Unknown restart policy: {other}, using default (on-failure)");
            RestartPolicy::default()
        }
    }
}

fn run_action(action: ServiceAction, dry_run: bool) -> svc_mgr::Result<ActionOutput> {
    if dry_run {
        for cmd in action.commands() {
            println!("{cmd}");
        }
        Ok(ActionOutput::None)
    } else {
        action.exec()
    }
}

fn main() {
    env_logger::init();
    let cli = Cli::parse();

    if let Err(e) = run(cli) {
        eprintln!("Error: {e}");
        process::exit(1);
    }
}

fn run(cli: Cli) -> svc_mgr::Result<()> {
    let mut manager = match cli.backend {
        Some(b) => TypedServiceManager::target(b.into()),
        None => TypedServiceManager::native()?,
    };

    if cli.user {
        manager.set_level(ServiceLevel::User)?;
    }

    match cli.command {
        Commands::Install {
            label,
            program,
            args,
            workdir,
            env,
            username,
            description,
            autostart,
            restart,
            restart_delay,
            max_retries,
        } => {
            let policy = parse_restart_policy(&restart, restart_delay, max_retries);
            let mut builder = ServiceBuilder::new(&label)?
                .program(program)
                .autostart(autostart)
                .restart_policy(policy);
            if !args.is_empty() {
                builder = builder.args(args);
            }
            if let Some(dir) = workdir {
                builder = builder.working_directory(dir);
            }
            for kv in env {
                if let Some((k, v)) = kv.split_once('=') {
                    builder = builder.env(k, v);
                } else {
                    eprintln!("Ignoring invalid env var (expected KEY=VALUE): {kv}");
                }
            }
            if let Some(u) = username {
                builder = builder.username(u);
            }
            if let Some(d) = description {
                builder = builder.description(d);
            }
            let config = builder.build()?;
            let action = manager.install(&config)?;
            run_action(action, cli.dry_run)?;
        }
        Commands::Uninstall { label } => {
            let label = label.parse()?;
            run_action(manager.uninstall(&label)?, cli.dry_run)?;
        }
        Commands::Start { label } => {
            let label = label.parse()?;
            run_action(manager.start(&label)?, cli.dry_run)?;
        }
        Commands::Stop { label } => {
            let label = label.parse()?;
            run_action(manager.stop(&label)?, cli.dry_run)?;
        }
        Commands::Restart { label } => {
            let label = label.parse()?;
            run_action(manager.restart(&label)?, cli.dry_run)?;
        }
        Commands::Status { label } => {
            let label = label.parse()?;
            let action = manager.status(&label)?;
            let output = run_action(action, cli.dry_run)?;
            if !cli.dry_run {
                match output.into_status() {
                    ServiceStatus::Running => println!("Running"),
                    ServiceStatus::Stopped(reason) => match reason {
                        Some(msg) => println!("Stopped: {msg}"),
                        None => println!("Stopped"),
                    },
                    ServiceStatus::NotInstalled => println!("Not installed"),
                }
            }
        }
        Commands::List => {
            let action = manager.list()?;
            let output = run_action(action, cli.dry_run)?;
            if !cli.dry_run {
                for name in output.into_list() {
                    println!("{name}");
                }
            }
        }
    }

    Ok(())
}
