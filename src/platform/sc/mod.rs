pub mod config;
pub mod shell_escape;

use crate::action::{ActionOutput, CmdOutput, ServiceAction};
use crate::error::{Error, Result};
use crate::{ServiceConfig, ServiceLabel, ServiceLevel, ServiceManager, ServiceStatus};

use self::config::ScServiceConfig;

pub struct ScServiceManager;

impl ScServiceManager {
    pub fn new() -> Self {
        Self
    }
}

impl ServiceManager for ScServiceManager {
    fn available(&self) -> Result<bool> {
        Ok(which::which("sc.exe").is_ok() || which::which("sc").is_ok())
    }

    fn install(&self, config: &ServiceConfig) -> Result<ServiceAction> {
        let sc_config = ScServiceConfig::from_config(config);
        let args = sc_config.to_create_args();
        Ok(ServiceAction::new().cmd("sc.exe", &args))
    }

    fn uninstall(&self, label: &ServiceLabel) -> Result<ServiceAction> {
        let name = label.to_qualified_name();
        Ok(ServiceAction::new().cmd("sc.exe", ["delete", &name]))
    }

    fn start(&self, label: &ServiceLabel) -> Result<ServiceAction> {
        let name = label.to_qualified_name();
        Ok(ServiceAction::new().cmd("sc.exe", ["start", &name]))
    }

    fn stop(&self, label: &ServiceLabel) -> Result<ServiceAction> {
        let name = label.to_qualified_name();
        Ok(ServiceAction::new().cmd("sc.exe", ["stop", &name]))
    }

    fn status(&self, label: &ServiceLabel) -> Result<ServiceAction> {
        let name = label.to_qualified_name();
        Ok(ServiceAction::new()
            .cmd_ignore_error("sc.exe", ["query", &name])
            .with_parser(|outputs: &[CmdOutput]| {
                let out = outputs.last();
                match out {
                    None => Ok(ActionOutput::Status(ServiceStatus::NotInstalled)),
                    Some(o) => match o.exit_code {
                        Some(1060) => Ok(ActionOutput::Status(ServiceStatus::NotInstalled)),
                        _ => {
                            if o.stdout.contains("RUNNING") {
                                Ok(ActionOutput::Status(ServiceStatus::Running))
                            } else if o.stdout.contains("STOPPED") {
                                Ok(ActionOutput::Status(ServiceStatus::Stopped(None)))
                            } else if o.exit_code != Some(0) {
                                Ok(ActionOutput::Status(ServiceStatus::NotInstalled))
                            } else {
                                Ok(ActionOutput::Status(ServiceStatus::Stopped(
                                    Some(o.stdout.clone()),
                                )))
                            }
                        }
                    },
                }
            }))
    }

    fn level(&self) -> ServiceLevel {
        ServiceLevel::System
    }

    fn set_level(&mut self, level: ServiceLevel) -> Result<()> {
        if level != ServiceLevel::System {
            return Err(Error::Unsupported(
                "sc.exe only supports system-level services".into(),
            ));
        }
        Ok(())
    }

    fn list(&self) -> Result<ServiceAction> {
        Ok(ServiceAction::new()
            .cmd_ignore_error("sc.exe", ["query", "type=", "service", "state=", "all"])
            .with_parser(|outputs: &[CmdOutput]| {
                let mut services = Vec::new();
                if let Some(out) = outputs.last() {
                    for line in out.stdout.lines() {
                        let line = line.trim();
                        if let Some(name) = line.strip_prefix("SERVICE_NAME:") {
                            let name = name.trim();
                            if !name.is_empty() {
                                services.push(name.to_string());
                            }
                        }
                    }
                }
                Ok(ActionOutput::List(services))
            }))
    }
}
