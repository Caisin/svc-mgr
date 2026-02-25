use svc_mgr::{RestartPolicy, ServiceBuilder};

#[test]
fn builder_basic() {
    let config = ServiceBuilder::new("com.example.myapp")
        .unwrap()
        .program("/usr/bin/myapp")
        .build()
        .unwrap();

    assert_eq!(config.label.to_qualified_name(), "com.example.myapp");
    assert_eq!(config.program.to_str().unwrap(), "/usr/bin/myapp");
    assert!(config.args.is_empty());
    assert!(!config.autostart);
}

#[test]
fn builder_full_config() {
    let config = ServiceBuilder::new("com.example.myapp")
        .unwrap()
        .program("/usr/bin/myapp")
        .args(["--port", "8080"])
        .working_directory("/opt/myapp")
        .env("RUST_LOG", "info")
        .env("APP_ENV", "production")
        .description("My Application Service")
        .username("myapp")
        .autostart(true)
        .restart_on_failure(5, 3)
        .build()
        .unwrap();

    assert_eq!(config.args.len(), 2);
    assert_eq!(
        config.working_directory.unwrap().to_str().unwrap(),
        "/opt/myapp"
    );
    assert_eq!(config.environment.len(), 2);
    assert_eq!(config.environment[0], ("RUST_LOG".into(), "info".into()));
    assert_eq!(config.username.as_deref(), Some("myapp"));
    assert_eq!(config.description.as_deref(), Some("My Application Service"));
    assert!(config.autostart);
    assert_eq!(
        config.restart_policy,
        RestartPolicy::OnFailure {
            delay_secs: Some(5),
            max_retries: Some(3),
            reset_after_secs: None,
        }
    );
}

#[test]
fn builder_missing_program_fails() {
    let result = ServiceBuilder::new("com.example.myapp")
        .unwrap()
        .build();
    assert!(result.is_err());
}

#[test]
fn builder_invalid_label_fails() {
    let result = ServiceBuilder::new("");
    assert!(result.is_err());
}

#[test]
fn builder_with_contents_override() {
    let config = ServiceBuilder::new("myapp")
        .unwrap()
        .program("/usr/bin/myapp")
        .contents("[Unit]\nDescription=custom\n")
        .build()
        .unwrap();

    assert_eq!(
        config.contents.as_deref(),
        Some("[Unit]\nDescription=custom\n")
    );
}

#[test]
fn builder_default_restart_policy() {
    let config = ServiceBuilder::new("myapp")
        .unwrap()
        .program("/usr/bin/myapp")
        .build()
        .unwrap();

    assert_eq!(
        config.restart_policy,
        RestartPolicy::OnFailure {
            delay_secs: None,
            max_retries: None,
            reset_after_secs: None,
        }
    );
}

#[test]
fn builder_custom_restart_policy() {
    let config = ServiceBuilder::new("myapp")
        .unwrap()
        .program("/usr/bin/myapp")
        .restart_policy(RestartPolicy::Always {
            delay_secs: Some(10),
        })
        .build()
        .unwrap();

    assert_eq!(
        config.restart_policy,
        RestartPolicy::Always {
            delay_secs: Some(10)
        }
    );
}
