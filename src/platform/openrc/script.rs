use crate::ServiceConfig;

/// Typed representation of an OpenRC init script.
#[derive(Debug, Clone)]
pub struct OpenRcScript {
    pub name: String,
    pub description: String,
    pub command: String,
    pub command_args: String,
    pub pidfile: String,
    pub command_background: bool,
    pub depend: Vec<String>,
    pub output_log: Option<String>,
    pub error_log: Option<String>,
}

impl OpenRcScript {
    pub fn from_config(config: &ServiceConfig) -> Self {
        let name = config.label.to_script_name();
        let command_args = config
            .args
            .iter()
            .map(|a| a.to_string_lossy().into_owned())
            .collect::<Vec<_>>()
            .join(" ");

        let stdout_path = config.stdout_file.as_ref()
            .map(|p| p.to_string_lossy().into_owned());
        let stderr_path = config.stderr_file.as_ref()
            .map(|p| p.to_string_lossy().into_owned());

        Self {
            description: config
                .description
                .clone()
                .unwrap_or_else(|| name.clone()),
            command: config.program.to_string_lossy().into_owned(),
            command_args,
            pidfile: format!("/run/{}.pid", name),
            command_background: true,
            depend: vec![name.clone()],
            output_log: stdout_path.clone(),
            error_log: stderr_path.or(stdout_path),
            name,
        }
    }

    pub fn render(&self) -> String {
        let mut out = String::new();
        out.push_str("#!/sbin/openrc-run\n\n");
        out.push_str(&format!("description=\"{}\"\n", self.description));
        out.push_str(&format!("command=\"{}\"\n", self.command));
        if !self.command_args.is_empty() {
            out.push_str(&format!("command_args=\"{}\"\n", self.command_args));
        }
        out.push_str(&format!("pidfile=\"{}\"\n", self.pidfile));
        if self.command_background {
            out.push_str("command_background=true\n");
        }
        if let Some(path) = &self.output_log {
            out.push_str(&format!("output_log=\"{}\"\n", path));
        }
        if let Some(path) = &self.error_log {
            out.push_str(&format!("error_log=\"{}\"\n", path));
        }
        out.push('\n');
        out.push_str("depend() {\n");
        for dep in &self.depend {
            out.push_str(&format!("\tprovide {}\n", dep));
        }
        out.push_str("}\n");
        out
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
            working_directory: Some(PathBuf::from("/opt/myapp")),
            environment: vec![("RUST_LOG".into(), "info".into())],
            username: Some("myapp".into()),
            description: Some("My Application".into()),
            autostart: true,
            restart_policy: crate::RestartPolicy::default(),
            contents: None,
            stdout_file: None,
            stderr_file: None,
        }
    }

    #[test]
    fn render_full_script() {
        let script = OpenRcScript::from_config(&test_config());
        let output = script.render();

        assert!(output.starts_with("#!/sbin/openrc-run"));
        assert!(output.contains("description=\"My Application\""));
        assert!(output.contains("command=\"/usr/bin/myapp\""));
        assert!(output.contains("command_args=\"--port 8080\""));
        assert!(output.contains("pidfile=\"/run/example-myapp.pid\""));
        assert!(output.contains("command_background=true"));
        assert!(output.contains("depend()"));
        assert!(output.contains("provide example-myapp"));
    }

    #[test]
    fn render_minimal_no_args() {
        let mut config = test_config();
        config.args.clear();
        config.description = None;
        config.label = "myapp".parse().unwrap();

        let script = OpenRcScript::from_config(&config);
        let output = script.render();

        assert!(output.contains("command=\"/usr/bin/myapp\""));
        assert!(!output.contains("command_args"));
        assert_eq!(script.description, "myapp");
    }
}
