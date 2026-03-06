use std::ffi::OsStr;
use std::path::Path;
use std::process::{Command, Output, Stdio};

use crate::error::{Error, Result};

/// Write data to a file, creating parent directories as needed.
/// On Unix, sets the file permissions to `mode`.
pub fn write_file(path: &Path, data: &[u8], mode: u32) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| Error::FileError {
            path: parent.to_path_buf(),
            source: e,
        })?;
    }

    #[cfg(unix)]
    {
        use std::fs::OpenOptions;
        use std::io::Write;
        use std::os::unix::fs::OpenOptionsExt;

        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .mode(mode)
            .open(path)
            .map_err(|e| Error::FileError {
                path: path.to_path_buf(),
                source: e,
            })?;
        file.write_all(data).map_err(|e| Error::FileError {
            path: path.to_path_buf(),
            source: e,
        })?;
    }

    #[cfg(not(unix))]
    {
        std::fs::write(path, data).map_err(|e| Error::FileError {
            path: path.to_path_buf(),
            source: e,
        })?;
    }

    Ok(())
}

/// Execute a command and return its output, converting failures to Error.
pub fn execute_command(program: &str, args: &[impl AsRef<OsStr>]) -> Result<Output> {
    let output = Command::new(program)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .map_err(Error::Io)?;

    Ok(output)
}

/// Execute a command and return error if it fails (non-zero exit).
pub fn run_command(program: &str, args: &[impl AsRef<OsStr>]) -> Result<Output> {
    let output = execute_command(program, args)?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        let message = if stderr.is_empty() {
            stdout.to_string()
        } else {
            stderr.to_string()
        };
        return Err(Error::CommandFailed {
            command: program.to_string(),
            code: output.status.code().unwrap_or(-1),
            message,
        });
    }
    Ok(output)
}

pub(crate) fn format_command_preview(program: &str, args: &[String]) -> String {
    let mut parts = Vec::with_capacity(args.len() + 1);
    parts.push(quote_preview_arg(program));
    parts.extend(args.iter().map(|arg| quote_preview_arg(arg)));
    parts.join(" ")
}

#[cfg(unix)]
pub(crate) fn quote_preview_arg(arg: &str) -> String {
    if !needs_unix_quotes(arg) {
        return arg.to_string();
    }

    format!("'{}'", arg.replace('\'', r#"'\''"#))
}

#[cfg(unix)]
fn needs_unix_quotes(arg: &str) -> bool {
    arg.is_empty()
        || arg.bytes().any(|byte| {
            !matches!(
                byte,
                b'a'..=b'z'
                    | b'A'..=b'Z'
                    | b'0'..=b'9'
                    | b'/'
                    | b'.'
                    | b'_'
                    | b'-'
                    | b':'
                    | b'='
                    | b'@'
                    | b'+'
                    | b','
            )
        })
}

#[cfg(windows)]
pub(crate) fn quote_preview_arg(arg: &str) -> String {
    if arg.is_empty() {
        return "\"\"".to_string();
    }

    if !arg.contains([' ', '\t', '"']) {
        return arg.to_string();
    }

    format!("\"{}\"", arg.replace('"', r#"\""#))
}
