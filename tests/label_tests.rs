use svc_mgr::ServiceLabel;

#[test]
fn parse_single_token() {
    let label: ServiceLabel = "myapp".parse().unwrap();
    assert_eq!(label.qualifier, None);
    assert_eq!(label.organization, None);
    assert_eq!(label.application, "myapp");
}

#[test]
fn parse_two_tokens() {
    let label: ServiceLabel = "example.myapp".parse().unwrap();
    assert_eq!(label.qualifier, None);
    assert_eq!(label.organization.as_deref(), Some("example"));
    assert_eq!(label.application, "myapp");
}

#[test]
fn parse_three_tokens() {
    let label: ServiceLabel = "com.example.myapp".parse().unwrap();
    assert_eq!(label.qualifier.as_deref(), Some("com"));
    assert_eq!(label.organization.as_deref(), Some("example"));
    assert_eq!(label.application, "myapp");
}

#[test]
fn parse_four_tokens_joins_rest() {
    let label: ServiceLabel = "com.example.my.app".parse().unwrap();
    assert_eq!(label.qualifier.as_deref(), Some("com"));
    assert_eq!(label.organization.as_deref(), Some("example"));
    assert_eq!(label.application, "my.app");
}

#[test]
fn parse_empty_label_fails() {
    let result: Result<ServiceLabel, _> = "".parse();
    assert!(result.is_err());
}

#[test]
fn parse_whitespace_only_fails() {
    let result: Result<ServiceLabel, _> = "   ".parse();
    assert!(result.is_err());
}

#[test]
fn qualified_name_single() {
    let label = ServiceLabel::new("myapp");
    assert_eq!(label.to_qualified_name(), "myapp");
}

#[test]
fn qualified_name_full() {
    let label: ServiceLabel = "com.example.myapp".parse().unwrap();
    assert_eq!(label.to_qualified_name(), "com.example.myapp");
}

#[test]
fn script_name_with_org() {
    let label: ServiceLabel = "com.example.myapp".parse().unwrap();
    assert_eq!(label.to_script_name(), "example-myapp");
}

#[test]
fn script_name_without_org() {
    let label = ServiceLabel::new("myapp");
    assert_eq!(label.to_script_name(), "myapp");
}

#[test]
fn display_uses_qualified_name() {
    let label: ServiceLabel = "com.example.myapp".parse().unwrap();
    assert_eq!(format!("{}", label), "com.example.myapp");
}

#[test]
fn label_equality() {
    let a: ServiceLabel = "com.example.myapp".parse().unwrap();
    let b: ServiceLabel = "com.example.myapp".parse().unwrap();
    assert_eq!(a, b);
}

#[test]
fn label_inequality() {
    let a: ServiceLabel = "com.example.app1".parse().unwrap();
    let b: ServiceLabel = "com.example.app2".parse().unwrap();
    assert_ne!(a, b);
}
