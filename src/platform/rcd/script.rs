use crate::ServiceConfig;

/// Typed representation of a FreeBSD rc.d script.
#[derive(Debug, Clone)]
pub struct RcdScript {
    pub name: String,
    pub rcvar: String,
    pub command: String,
    pub pidfile: String,
    pub daemon_args: String,
}

impl RcdScript {
    pub fn from_config(config: &ServiceConfig) -> Self {
        let name = config.label.to_script_name();
        let daemon_args = config
            .args
            .iter()
            .map(|a| a.to_string_lossy().into_owned())
            .collect::<Vec<_>>()
            .join(" ");

        Self {
            rcvar: format!("{}_enable", name),
            command: config.program.to_string_lossy().into_owned(),
            pidfile: format!("/var/run/{}.pid", name),
            daemon_args,
            name,
        }
    }

    pub fn render(&self) -> String {
        let mut out = String::new();
        out.push_str("#!/bin/sh\n\n");
        out.push_str(&format!("# PROVIDE: {}\n", self.name));
        out.push_str("# REQUIRE: LOGIN FILESYSTEMS\n");
        out.push_str("# KEYWORD: shutdown\n\n");
        out.push_str(". /etc/rc.subr\n\n");
        out.push_str(&format!("name=\"{}\"\n", self.name));
        out.push_str(&format!("rcvar=\"{}\"\n", self.rcvar));
        out.push_str(&format!("command=\"{}\"\n", self.command));
        out.push_str(&format!("pidfile=\"{}\"\n", self.pidfile));
        if !self.daemon_args.is_empty() {
            out.push_str(&format!(
                "command_args=\"-p ${{pidfile}} {} {}\"\n",
                self.command, self.daemon_args
            ));
        } else {
            out.push_str(&format!(
                "command_args=\"-p ${{pidfile}} {}\"\n",
                self.command
            ));
        }
        out.push_str("command=\"/usr/sbin/daemon\"\n\n");
        out.push_str("load_rc_config $name\n");
        out.push_str("run_rc_command \"$1\"\n");
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
            working_directory: None,
            environment: vec![],
            username: None,
            description: None,
            autostart: false,
            restart_policy: crate::RestartPolicy::Never,
            contents: None,
        }
    }

    #[test]
    fn render_full_script() {
        let script = RcdScript::from_config(&test_config());
        let output = script.render();

        assert!(output.starts_with("#!/bin/sh"));
        assert!(output.contains("# PROVIDE: example-myapp"));
        assert!(output.contains("# REQUIRE: LOGIN FILESYSTEMS"));
        assert!(output.contains("name=\"example-myapp\""));
        assert!(output.contains("rcvar=\"example-myapp_enable\""));
        assert!(output.contains("pidfile=\"/var/run/example-myapp.pid\""));
        assert!(output.contains("command=\"/usr/sbin/daemon\""));
        assert!(output.contains("load_rc_config $name"));
    }

    #[test]
    fn render_minimal_no_args() {
        let mut config = test_config();
        config.args.clear();
        config.label = "myapp".parse().unwrap();

        let script = RcdScript::from_config(&config);
        let output = script.render();

        assert!(output.contains("command_args=\"-p ${pidfile} /usr/bin/myapp\""));
    }
}
