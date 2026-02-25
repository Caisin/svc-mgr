pub mod plist;

use std::path::PathBuf;

use crate::action::{ActionOutput, CmdOutput, ServiceAction};
use crate::error::{Error, Result};
use crate::{ServiceConfig, ServiceLabel, ServiceLevel, ServiceManager, ServiceStatus};

use self::plist::LaunchdPlist;

pub struct LaunchdServiceManager {
    user: bool,
}

impl LaunchdServiceManager {
    pub fn system() -> Self {
        Self { user: false }
    }

    pub fn user() -> Self {
        Self { user: true }
    }

    fn service_dir(&self) -> Result<PathBuf> {
        if self.user {
            let home = dirs::home_dir().ok_or_else(|| {
                Error::Io(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "home directory not found",
                ))
            })?;
            Ok(home.join("Library/LaunchAgents"))
        } else {
            Ok(PathBuf::from("/Library/LaunchDaemons"))
        }
    }

    fn plist_path(&self, label: &ServiceLabel) -> Result<PathBuf> {
        let dir = self.service_dir()?;
        Ok(dir.join(format!("{}.plist", label.to_qualified_name())))
    }

    fn domain_target(&self) -> String {
        if self.user {
            let uid = std::process::Command::new("id")
                .arg("-u")
                .output()
                .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
                .unwrap_or_else(|_| "501".to_string());
            format!("gui/{}", uid)
        } else {
            "system".to_string()
        }
    }
}

impl ServiceManager for LaunchdServiceManager {
    fn available(&self) -> Result<bool> {
        Ok(which::which("launchctl").is_ok())
    }

    fn install(&self, config: &ServiceConfig) -> Result<ServiceAction> {
        let path = self.plist_path(&config.label)?;
        let data = if let Some(contents) = &config.contents {
            contents.as_bytes().to_vec()
        } else {
            let plist_obj = LaunchdPlist::from_config(config);
            plist_obj.render()?
        };
        let mut action = ServiceAction::new().write_file(&path, data, 0o644);
        if config.autostart {
            let domain = self.domain_target();
            action = action.cmd_ignore_error(
                "launchctl",
                ["bootstrap", &domain, &path.to_string_lossy()],
            );
        }
        Ok(action)
    }

    fn uninstall(&self, label: &ServiceLabel) -> Result<ServiceAction> {
        let path = self.plist_path(label)?;
        let qualified = label.to_qualified_name();
        let domain = self.domain_target();
        let service_target = format!("{}/{}", domain, qualified);
        Ok(ServiceAction::new()
            .cmd_ignore_error("launchctl", ["bootout", &service_target])
            .remove_file(&path))
    }

    fn start(&self, label: &ServiceLabel) -> Result<ServiceAction> {
        let qualified = label.to_qualified_name();
        let domain = self.domain_target();
        let service_target = format!("{}/{}", domain, qualified);
        let path = self.plist_path(label)?;
        Ok(ServiceAction::new()
            .cmd_ignore_error(
                "launchctl",
                ["bootstrap", &domain, &path.to_string_lossy()],
            )
            .cmd("launchctl", ["kickstart", "-k", &service_target]))
    }

    fn stop(&self, label: &ServiceLabel) -> Result<ServiceAction> {
        let qualified = label.to_qualified_name();
        let domain = self.domain_target();
        let service_target = format!("{}/{}", domain, qualified);
        Ok(ServiceAction::new().cmd(
            "launchctl",
            ["kill", "SIGTERM", &service_target],
        ))
    }

    fn status(&self, label: &ServiceLabel) -> Result<ServiceAction> {
        let qualified = label.to_qualified_name();
        let domain = self.domain_target();
        let target = format!("{}/{}", domain, qualified);
        Ok(ServiceAction::new()
            .cmd_ignore_error("launchctl", ["print", &target])
            .with_parser(|outputs: &[CmdOutput]| {
                let out = outputs.last();
                match out {
                    None => Ok(ActionOutput::Status(ServiceStatus::NotInstalled)),
                    Some(o) => {
                        if o.exit_code != Some(0) {
                            Ok(ActionOutput::Status(ServiceStatus::NotInstalled))
                        } else if o.stdout.contains("state = running") {
                            Ok(ActionOutput::Status(ServiceStatus::Running))
                        } else {
                            Ok(ActionOutput::Status(ServiceStatus::Stopped(None)))
                        }
                    }
                }
            }))
    }

    fn level(&self) -> ServiceLevel {
        if self.user {
            ServiceLevel::User
        } else {
            ServiceLevel::System
        }
    }

    fn set_level(&mut self, level: ServiceLevel) -> Result<()> {
        self.user = level == ServiceLevel::User;
        Ok(())
    }

    fn list(&self) -> Result<ServiceAction> {
        Ok(ServiceAction::new()
            .cmd_ignore_error("launchctl", ["list"])
            .with_parser(|outputs: &[CmdOutput]| {
                let mut services = Vec::new();
                if let Some(out) = outputs.last() {
                    for line in out.stdout.lines().skip(1) {
                        if let Some(label) = line.split('\t').nth(2) {
                            let label = label.trim();
                            if !label.is_empty() {
                                services.push(label.to_string());
                            }
                        }
                    }
                }
                Ok(ActionOutput::List(services))
            }))
    }
}
