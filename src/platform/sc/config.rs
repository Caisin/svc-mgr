use crate::ServiceConfig;

use super::shell_escape;

/// Windows service type for sc.exe.
#[derive(Debug, Clone, Copy, Default)]
pub enum WindowsServiceType {
    #[default]
    Own,
    Share,
    Kernel,
    FileSys,
    Rec,
}

impl WindowsServiceType {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Own => "own",
            Self::Share => "share",
            Self::Kernel => "kernel",
            Self::FileSys => "filesys",
            Self::Rec => "rec",
        }
    }
}

/// Windows start type for sc.exe.
#[derive(Debug, Clone, Copy, Default)]
pub enum WindowsStartType {
    Boot,
    System,
    #[default]
    Auto,
    Demand,
    Disabled,
}

impl WindowsStartType {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Boot => "boot",
            Self::System => "system",
            Self::Auto => "auto",
            Self::Demand => "demand",
            Self::Disabled => "disabled",
        }
    }
}

/// Windows error severity for sc.exe.
#[derive(Debug, Clone, Copy, Default)]
pub enum WindowsErrorSeverity {
    #[default]
    Normal,
    Severe,
    Critical,
    Ignore,
}

impl WindowsErrorSeverity {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Normal => "normal",
            Self::Severe => "severe",
            Self::Critical => "critical",
            Self::Ignore => "ignore",
        }
    }
}

/// Typed representation of sc.exe service configuration.
#[derive(Debug, Clone)]
pub struct ScServiceConfig {
    pub service_name: String,
    pub service_type: WindowsServiceType,
    pub start_type: WindowsStartType,
    pub error_severity: WindowsErrorSeverity,
    pub binpath: String,
    pub display_name: Option<String>,
}

impl ScServiceConfig {
    pub fn from_config(config: &ServiceConfig) -> Self {
        let args: Vec<String> = config
            .args
            .iter()
            .map(|a| a.to_string_lossy().into_owned())
            .collect();

        let binpath = shell_escape::build_binpath(
            &config.program.to_string_lossy(),
            &args,
        );

        Self {
            service_name: config.label.to_qualified_name(),
            service_type: WindowsServiceType::default(),
            start_type: if config.autostart {
                WindowsStartType::Auto
            } else {
                WindowsStartType::Demand
            },
            error_severity: WindowsErrorSeverity::default(),
            binpath,
            display_name: config.description.clone(),
        }
    }

    /// Build sc.exe create arguments.
    pub fn to_create_args(&self) -> Vec<String> {
        let mut args = vec![
            "create".to_string(),
            self.service_name.clone(),
            format!("type={}", self.service_type.as_str()),
            format!("start={}", self.start_type.as_str()),
            format!("error={}", self.error_severity.as_str()),
            format!("binpath={}", self.binpath),
        ];

        if let Some(name) = &self.display_name {
            args.push(format!("displayname={}", name));
        }

        args
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::OsString;
    use std::path::PathBuf;

    fn test_config() -> ServiceConfig {
        ServiceConfig {
            label: "com.example.myapp".parse().unwrap(),
            program: PathBuf::from("/usr/bin/myapp"),
            args: vec![OsString::from("--port"), OsString::from("8080")],
            working_directory: None,
            environment: vec![],
            username: None,
            description: Some("My Application".into()),
            autostart: true,
            restart_policy: crate::RestartPolicy::default(),
            contents: None,
        }
    }

    #[test]
    fn from_config_basic() {
        let sc = ScServiceConfig::from_config(&test_config());
        assert_eq!(sc.service_name, "com.example.myapp");
        assert!(sc.binpath.contains("/usr/bin/myapp"));
        assert!(sc.binpath.contains("--port"));
        assert_eq!(sc.start_type.as_str(), "auto");
        assert_eq!(sc.display_name.as_deref(), Some("My Application"));
    }

    #[test]
    fn demand_when_no_autostart() {
        let mut config = test_config();
        config.autostart = false;
        let sc = ScServiceConfig::from_config(&config);
        assert_eq!(sc.start_type.as_str(), "demand");
    }

    #[test]
    fn to_create_args_structure() {
        let sc = ScServiceConfig::from_config(&test_config());
        let args = sc.to_create_args();

        assert_eq!(args[0], "create");
        assert_eq!(args[1], "com.example.myapp");
        assert!(args.iter().any(|a| a.starts_with("type=")));
        assert!(args.iter().any(|a| a.starts_with("start=")));
        assert!(args.iter().any(|a| a.starts_with("error=")));
        assert!(args.iter().any(|a| a.starts_with("binpath=")));
        assert!(args.iter().any(|a| a.starts_with("displayname=")));
    }
}
