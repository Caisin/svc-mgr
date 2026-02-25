use std::fmt;
use std::str::FromStr;

use crate::error::{Error, Result};

/// A structured service label with optional qualifier and organization.
///
/// Parsing rules (dot-separated):
/// - 1 token:  application only
/// - 2 tokens: organization.application
/// - 3+ tokens: qualifier.organization.rest (rest joined by dots)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ServiceLabel {
    pub qualifier: Option<String>,
    pub organization: Option<String>,
    pub application: String,
}

impl ServiceLabel {
    pub fn new(application: impl Into<String>) -> Self {
        Self {
            qualifier: None,
            organization: None,
            application: application.into(),
        }
    }

    /// Fully qualified name: `qualifier.organization.application`
    /// Used by launchd, sc.exe, winsw.
    pub fn to_qualified_name(&self) -> String {
        let mut parts = Vec::new();
        if let Some(q) = &self.qualifier {
            parts.push(q.as_str());
        }
        if let Some(o) = &self.organization {
            parts.push(o.as_str());
        }
        parts.push(&self.application);
        parts.join(".")
    }

    /// Script-friendly name: `organization-application`
    /// Used by systemd, openrc, rcd.
    pub fn to_script_name(&self) -> String {
        match &self.organization {
            Some(org) => format!("{}-{}", org, self.application),
            None => self.application.clone(),
        }
    }
}

impl FromStr for ServiceLabel {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        let s = s.trim();
        if s.is_empty() {
            return Err(Error::InvalidLabel("empty label".into()));
        }

        let parts: Vec<&str> = s.split('.').collect();
        match parts.len() {
            1 => Ok(Self {
                qualifier: None,
                organization: None,
                application: parts[0].to_string(),
            }),
            2 => Ok(Self {
                qualifier: None,
                organization: Some(parts[0].to_string()),
                application: parts[1].to_string(),
            }),
            _ => Ok(Self {
                qualifier: Some(parts[0].to_string()),
                organization: Some(parts[1].to_string()),
                application: parts[2..].join("."),
            }),
        }
    }
}

impl fmt::Display for ServiceLabel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_qualified_name())
    }
}
