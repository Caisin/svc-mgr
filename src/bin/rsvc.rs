use std::process;

use clap::{Parser, Subcommand, ValueEnum};
use svc_mgr::{
    ActionOutput, RestartPolicy, ServiceAction, ServiceBuilder, ServiceLevel, ServiceManager,
    ServiceManagerKind, ServiceStatus, TypedServiceManager,
};

fn get_editor() -> String {
    std::env::var("EDITOR")
        .or_else(|_| std::env::var("VISUAL"))
        .unwrap_or_else(|_| {
            #[cfg(target_os = "windows")]
            {
                "notepad".to_string()
            }
            #[cfg(not(target_os = "windows"))]
            {
                "vi".to_string()
            }
        })
}

fn open_editor(path: &str) -> svc_mgr::Result<()> {
    let editor = get_editor();
    let status = std::process::Command::new(&editor)
        .arg(path)
        .status()
        .map_err(svc_mgr::Error::Io)?;

    if !status.success() {
        return Err(svc_mgr::Error::CommandFailed {
            command: format!("{} {}", editor, path),
            code: status.code().unwrap_or(-1),
            message: "Editor exited with error".to_string(),
        });
    }
    Ok(())
}

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

#[derive(Clone, Copy, ValueEnum)]
enum Backend {
    Launchd,
    Systemd,
    Openrc,
    Rcd,
    Sc,
    Winsw,
}

impl From<Backend> for ServiceManagerKind {
    fn from(backend: Backend) -> Self {
        match backend {
            Backend::Launchd => Self::Launchd,
            Backend::Systemd => Self::Systemd,
            Backend::Openrc => Self::OpenRc,
            Backend::Rcd => Self::Rcd,
            Backend::Sc => Self::Sc,
            Backend::Winsw => Self::WinSw,
        }
    }
}

#[derive(Clone, Copy, ValueEnum)]
enum RestartArg {
    Never,
    Always,
    OnFailure,
    OnSuccess,
}

#[derive(Subcommand)]
#[allow(clippy::large_enum_variant)]
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
        #[arg(long, value_enum, default_value_t = RestartArg::OnFailure)]
        restart: RestartArg,
        /// Delay in seconds before restart
        #[arg(long)]
        restart_delay: Option<u32>,
        /// Maximum retry count (only for on-failure)
        #[arg(long)]
        max_retries: Option<u32>,
        /// Log file path (stdout and stderr)
        #[arg(long)]
        log: Option<String>,
        /// Stdout log file path (overrides --log for stdout)
        #[arg(long)]
        stdout_file: Option<String>,
        /// Stderr log file path (overrides --log for stderr)
        #[arg(long)]
        stderr_file: Option<String>,
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
    /// Get detailed service information
    Info {
        /// Service label
        label: String,
    },
    /// Edit service configuration file
    Edit {
        /// Service label
        label: String,
    },
    /// List installed services
    List,
}

fn parse_restart_policy(
    restart: RestartArg,
    delay: Option<u32>,
    max_retries: Option<u32>,
) -> RestartPolicy {
    match restart {
        RestartArg::Never => RestartPolicy::Never,
        RestartArg::Always => RestartPolicy::Always { delay_secs: delay },
        RestartArg::OnFailure => RestartPolicy::OnFailure {
            delay_secs: delay,
            max_retries,
            reset_after_secs: None,
        },
        RestartArg::OnSuccess => RestartPolicy::OnSuccess { delay_secs: delay },
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

    if let Err(err) = run(cli) {
        eprintln!("Error: {err}");
        process::exit(1);
    }
}

fn run(cli: Cli) -> svc_mgr::Result<()> {
    let mut manager = match cli.backend {
        Some(backend) => TypedServiceManager::target(backend.into())?,
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
            log,
            stdout_file,
            stderr_file,
        } => {
            let policy = parse_restart_policy(restart, restart_delay, max_retries);
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
                if let Some((key, value)) = kv.split_once('=') {
                    builder = builder.env(key, value);
                } else {
                    eprintln!("Ignoring invalid env var (expected KEY=VALUE): {kv}");
                }
            }
            if let Some(user) = username {
                builder = builder.username(user);
            }
            if let Some(desc) = description {
                builder = builder.description(desc);
            }
            match (log, stdout_file, stderr_file) {
                (Some(path), None, None) => builder = builder.log(path),
                (_, stdout_path, stderr_path) => {
                    if let Some(path) = stdout_path {
                        builder = builder.stdout_file(path);
                    }
                    if let Some(path) = stderr_path {
                        builder = builder.stderr_file(path);
                    }
                }
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
                match output.into_status()? {
                    ServiceStatus::Running => println!("Running"),
                    ServiceStatus::Stopped(reason) => match reason {
                        Some(message) => println!("Stopped: {message}"),
                        None => println!("Stopped"),
                    },
                    ServiceStatus::NotInstalled => println!("Not installed"),
                }
            }
        }
        Commands::Info { label } => {
            let label = label.parse()?;
            let action = manager.info(&label)?;
            let output = run_action(action, cli.dry_run)?;
            if !cli.dry_run {
                let info = output.into_info()?;
                if !info.config_path.is_empty() {
                    println!("{}", info.config_path);
                    println!("{}", "-".repeat(info.config_path.len()));
                }
                println!("{}", info.config_content);
            }
        }
        Commands::Edit { label } => {
            let label = label.parse()?;
            let action = manager.info(&label)?;

            if cli.dry_run {
                for cmd in action.commands() {
                    println!("{cmd}");
                }
                let editor = get_editor();
                println!("{} <config-file>", editor);
            } else {
                let output = action.exec()?;
                let info = output.into_info()?;

                if info.config_path.is_empty() {
                    eprintln!("Error: This backend does not use configuration files");
                    eprintln!("Hint: sc.exe services are configured via registry, use 'sc.exe config' instead");
                    process::exit(1);
                }

                open_editor(&info.config_path)?;
            }
        }
        Commands::List => {
            let action = manager.list()?;
            let output = run_action(action, cli.dry_run)?;
            if !cli.dry_run {
                for name in output.into_list()? {
                    println!("{name}");
                }
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_restart_policy_is_rejected() {
        let cli = Cli::try_parse_from([
            "rsvc",
            "install",
            "com.example.myapp",
            "--program",
            "/usr/bin/myapp",
            "--restart",
            "bogus",
        ]);

        assert!(cli.is_err());
    }
}
