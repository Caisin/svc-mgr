use crate::{RestartPolicy, ServiceConfig};

/// Typed representation of a WinSW XML service definition.
#[derive(Debug, Clone)]
pub struct WinSwXmlDef {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub executable: String,
    pub arguments: Option<String>,
    pub working_directory: Option<String>,
    pub environment: Vec<(String, String)>,
    pub onfailure: Vec<OnFailureAction>,
    pub reset_failure: Option<u32>,
    pub startmode: String,
    pub username: Option<String>,
    pub log_path: Option<String>,
}

#[derive(Debug, Clone)]
pub struct OnFailureAction {
    pub action: String,
    pub delay_ms: Option<u32>,
}

impl WinSwXmlDef {
    pub fn from_config(config: &ServiceConfig) -> Self {
        let arguments = if config.args.is_empty() {
            None
        } else {
            Some(
                config
                    .args
                    .iter()
                    .map(|a| a.to_string_lossy().into_owned())
                    .collect::<Vec<_>>()
                    .join(" "),
            )
        };

        let (onfailure, reset_failure) = match &config.restart_policy {
            RestartPolicy::Never => {
                (vec![OnFailureAction { action: "none".into(), delay_ms: None }], None)
            }
            RestartPolicy::Always { delay_secs } => {
                let delay_ms = delay_secs.map(|s| s * 1000);
                (vec![OnFailureAction { action: "restart".into(), delay_ms }], None)
            }
            RestartPolicy::OnFailure {
                delay_secs,
                max_retries,
                reset_after_secs,
            } => {
                let delay_ms = delay_secs.map(|s| s * 1000);
                let mut actions = Vec::new();
                let retries = max_retries.unwrap_or(1);
                for _ in 0..retries {
                    actions.push(OnFailureAction {
                        action: "restart".into(),
                        delay_ms,
                    });
                }
                actions.push(OnFailureAction {
                    action: "none".into(),
                    delay_ms: None,
                });
                (actions, *reset_after_secs)
            }
            RestartPolicy::OnSuccess { delay_secs } => {
                // WinSW doesn't distinguish on-success; fall back to restart
                log::warn!("WinSW does not support OnSuccess restart; using restart");
                let delay_ms = delay_secs.map(|s| s * 1000);
                (vec![OnFailureAction { action: "restart".into(), delay_ms }], None)
            }
        };

        let startmode = if config.autostart {
            "Automatic"
        } else {
            "Manual"
        };

        Self {
            id: config.label.to_qualified_name(),
            name: config.label.to_qualified_name(),
            description: config.description.clone(),
            executable: config.program.to_string_lossy().into_owned(),
            arguments,
            working_directory: config
                .working_directory
                .as_ref()
                .map(|p| p.to_string_lossy().into_owned()),
            environment: config.environment.clone(),
            onfailure,
            reset_failure,
            startmode: startmode.to_string(),
            username: config.username.clone(),
            log_path: config.stdout_file.as_ref()
                .map(|p| p.parent().unwrap_or(p).to_string_lossy().into_owned()),
        }
    }

    /// Render to WinSW XML format using quick-xml.
    #[cfg(target_os = "windows")]
    pub fn render(&self) -> crate::Result<String> {
        use quick_xml::events::{BytesEnd, BytesStart, BytesText, Event};
        use quick_xml::Writer;
        use std::io::Cursor;

        let mut writer = Writer::new(Cursor::new(Vec::new()));

        // <service>
        writer
            .write_event(Event::Start(BytesStart::new("service")))
            .map_err(|e| crate::Error::Io(std::io::Error::other(e)))?;

        let write_element = |w: &mut Writer<Cursor<Vec<u8>>>, tag: &str, text: &str| -> crate::Result<()> {
            w.write_event(Event::Start(BytesStart::new(tag)))
                .map_err(|e| crate::Error::Io(std::io::Error::other(e)))?;
            w.write_event(Event::Text(BytesText::new(text)))
                .map_err(|e| crate::Error::Io(std::io::Error::other(e)))?;
            w.write_event(Event::End(BytesEnd::new(tag)))
                .map_err(|e| crate::Error::Io(std::io::Error::other(e)))?;
            Ok(())
        };

        write_element(&mut writer, "id", &self.id)?;
        write_element(&mut writer, "name", &self.name)?;

        if let Some(desc) = &self.description {
            write_element(&mut writer, "description", desc)?;
        }

        write_element(&mut writer, "executable", &self.executable)?;

        if let Some(args) = &self.arguments {
            write_element(&mut writer, "arguments", args)?;
        }

        if let Some(wd) = &self.working_directory {
            write_element(&mut writer, "workingdirectory", wd)?;
        }

        for (k, v) in &self.environment {
            let mut elem = BytesStart::new("env");
            elem.push_attribute(("name", k.as_str()));
            elem.push_attribute(("value", v.as_str()));
            writer
                .write_event(Event::Empty(elem))
                .map_err(|e| crate::Error::Io(std::io::Error::other(e)))?;
        }

        for action in &self.onfailure {
            let mut elem = BytesStart::new("onfailure");
            elem.push_attribute(("action", action.action.as_str()));
            if let Some(delay) = action.delay_ms {
                elem.push_attribute(("delay", format!("{} ms", delay).as_str()));
            }
            writer
                .write_event(Event::Empty(elem))
                .map_err(|e| crate::Error::Io(std::io::Error::other(e)))?;
        }

        if let Some(reset) = self.reset_failure {
            write_element(&mut writer, "resetfailure", &format!("{} sec", reset))?;
        }

        write_element(&mut writer, "startmode", &self.startmode)?;

        if let Some(log_path) = &self.log_path {
            write_element(&mut writer, "logpath", log_path)?;
        }

        if let Some(user) = &self.username {
            writer
                .write_event(Event::Start(BytesStart::new("serviceaccount")))
                .map_err(|e| crate::Error::Io(std::io::Error::other(e)))?;
            write_element(&mut writer, "username", user)?;
            writer
                .write_event(Event::End(BytesEnd::new("serviceaccount")))
                .map_err(|e| crate::Error::Io(std::io::Error::other(e)))?;
        }

        // </service>
        writer
            .write_event(Event::End(BytesEnd::new("service")))
            .map_err(|e| crate::Error::Io(std::io::Error::other(e)))?;

        let result = writer.into_inner().into_inner();
        String::from_utf8(result)
            .map_err(|e| crate::Error::Io(std::io::Error::new(std::io::ErrorKind::InvalidData, e)))
    }
}
