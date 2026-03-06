use std::fmt;
use std::path::PathBuf;
use std::process::Output;
use std::sync::Arc;

use crate::error::{Error, Result};
use crate::{ServiceStatus, utils};

/// A single step in a service action.
#[derive(Debug, Clone)]
pub enum ActionStep {
    /// Write content to a file with given permissions.
    WriteFile {
        path: PathBuf,
        data: Vec<u8>,
        mode: u32,
    },
    /// Remove a file if it exists.
    RemoveFile { path: PathBuf },
    /// Read directory entries at execution time.
    ReadDir {
        path: PathBuf,
        extension: Option<String>,
    },
    /// Run a command, fail on non-zero exit.
    Cmd {
        program: String,
        args: Vec<String>,
    },
    /// Run a command, ignore errors.
    CmdIgnoreError {
        program: String,
        args: Vec<String>,
    },
}

impl fmt::Display for ActionStep {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::WriteFile { path, .. } => write!(f, "# write file: {}", path.display()),
            Self::RemoveFile { path } => write!(f, "rm {}", path.display()),
            Self::ReadDir { path, extension } => match extension {
                Some(extension) => write!(f, "# list dir: {} (*.{extension})", path.display()),
                None => write!(f, "# list dir: {}", path.display()),
            },
            Self::Cmd { program, args } | Self::CmdIgnoreError { program, args } => {
                write!(f, "{}", utils::format_command_preview(program, args))
            }
        }
    }
}

/// Raw output from a single command execution.
#[derive(Debug, Clone)]
pub struct CmdOutput {
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
}

impl From<Output> for CmdOutput {
    fn from(output: Output) -> Self {
        Self {
            exit_code: output.status.code(),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        }
    }
}

/// The result of executing a ServiceAction.
#[derive(Debug, Clone)]
pub enum ActionOutput {
    /// No meaningful output (install, uninstall, start, stop, restart).
    None,
    /// Service status query result.
    Status(ServiceStatus),
    /// Service list result.
    List(Vec<String>),
}

impl ActionOutput {
    /// Extract ServiceStatus.
    pub fn into_status(self) -> Result<ServiceStatus> {
        match self {
            Self::Status(status) => Ok(status),
            Self::None => Err(Error::UnexpectedActionOutput {
                expected: "Status",
                actual: "None",
            }),
            Self::List(_) => Err(Error::UnexpectedActionOutput {
                expected: "Status",
                actual: "List",
            }),
        }
    }

    /// Extract service list.
    pub fn into_list(self) -> Result<Vec<String>> {
        match self {
            Self::List(list) => Ok(list),
            Self::None => Err(Error::UnexpectedActionOutput {
                expected: "List",
                actual: "None",
            }),
            Self::Status(_) => Err(Error::UnexpectedActionOutput {
                expected: "List",
                actual: "Status",
            }),
        }
    }
}

type OutputParser = Arc<dyn Fn(&[CmdOutput]) -> Result<ActionOutput> + Send + Sync>;

/// A composable action returned by ServiceManager methods.
///
/// Call `.exec()` to execute locally, `.commands()` to preview,
/// or `.parse()` to interpret remote command outputs.
pub struct ServiceAction {
    steps: Vec<ActionStep>,
    parser: Option<OutputParser>,
}

impl fmt::Debug for ServiceAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ServiceAction")
            .field("steps", &self.steps)
            .field("has_parser", &self.parser.is_some())
            .finish()
    }
}

impl ServiceAction {
    pub fn new() -> Self {
        Self {
            steps: Vec::new(),
            parser: None,
        }
    }

    pub fn write_file(
        mut self,
        path: impl Into<PathBuf>,
        data: impl Into<Vec<u8>>,
        mode: u32,
    ) -> Self {
        self.steps.push(ActionStep::WriteFile {
            path: path.into(),
            data: data.into(),
            mode,
        });
        self
    }

    pub fn remove_file(mut self, path: impl Into<PathBuf>) -> Self {
        self.steps.push(ActionStep::RemoveFile { path: path.into() });
        self
    }

    pub fn read_dir<S>(mut self, path: impl Into<PathBuf>, extension: Option<S>) -> Self
    where
        S: Into<String>,
    {
        self.steps.push(ActionStep::ReadDir {
            path: path.into(),
            extension: extension.map(Into::into),
        });
        self
    }

    pub fn cmd(
        mut self,
        program: impl Into<String>,
        args: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        self.steps.push(ActionStep::Cmd {
            program: program.into(),
            args: args.into_iter().map(Into::into).collect(),
        });
        self
    }

    pub fn cmd_ignore_error(
        mut self,
        program: impl Into<String>,
        args: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        self.steps.push(ActionStep::CmdIgnoreError {
            program: program.into(),
            args: args.into_iter().map(Into::into).collect(),
        });
        self
    }

    /// Set an output parser for interpreting command results.
    pub fn with_parser(
        mut self,
        parser: impl Fn(&[CmdOutput]) -> Result<ActionOutput> + Send + Sync + 'static,
    ) -> Self {
        self.parser = Some(Arc::new(parser));
        self
    }

    /// Append all steps from another action (parser from self is kept).
    pub fn merge(mut self, other: ServiceAction) -> Self {
        self.steps.extend(other.steps);
        self
    }

    /// Get the steps for inspection.
    pub fn steps(&self) -> &[ActionStep] {
        &self.steps
    }

    /// Preview the commands as human-readable strings.
    pub fn commands(&self) -> Vec<String> {
        self.steps.iter().map(ToString::to_string).collect()
    }

    /// Parse remote command outputs using the stored parser.
    /// Use this when executing commands remotely (e.g. via SSH).
    pub fn parse(&self, outputs: &[CmdOutput]) -> Result<ActionOutput> {
        match &self.parser {
            Some(parser) => parser(outputs),
            None => Ok(ActionOutput::None),
        }
    }

    /// Execute all steps locally and return parsed output.
    pub fn exec(self) -> Result<ActionOutput> {
        let mut cmd_outputs = Vec::new();
        for step in &self.steps {
            match step {
                ActionStep::WriteFile { path, data, mode } => {
                    utils::write_file(path, data, *mode)?;
                }
                ActionStep::RemoveFile { path } => {
                    if path.exists() {
                        std::fs::remove_file(path).map_err(|e| Error::FileError {
                            path: path.clone(),
                            source: e,
                        })?;
                    }
                }
                ActionStep::ReadDir { path, extension } => {
                    let mut entries = Vec::new();
                    if path.exists() {
                        for entry in std::fs::read_dir(path).map_err(|e| Error::FileError {
                            path: path.clone(),
                            source: e,
                        })? {
                            let entry = entry.map_err(Error::Io)?;
                            let entry_path = entry.path();
                            if let Some(extension) = extension
                                && entry_path.extension().and_then(|ext| ext.to_str())
                                    != Some(extension.as_str())
                            {
                                continue;
                            }
                            if let Some(name) = entry.file_name().to_str() {
                                entries.push(name.to_string());
                            }
                        }
                    }
                    entries.sort();
                    cmd_outputs.push(CmdOutput {
                        exit_code: Some(0),
                        stdout: entries.join("\n"),
                        stderr: String::new(),
                    });
                }
                ActionStep::Cmd { program, args } => {
                    let output = utils::run_command(program, args)?;
                    cmd_outputs.push(CmdOutput::from(output));
                }
                ActionStep::CmdIgnoreError { program, args } => {
                    if let Ok(output) = utils::execute_command(program, args) {
                        cmd_outputs.push(CmdOutput::from(output));
                    }
                }
            }
        }
        self.parse(&cmd_outputs)
    }
}

impl Default for ServiceAction {
    fn default() -> Self {
        Self::new()
    }
}
