use std::ffi::OsStr;
use std::path::Path;
use std::process::{Command, Output, Stdio};

use crate::error::{Error, Result};

/// Write data to a file, creating parent directories as needed.
/// On Unix, sets the file permissions to `mode`.
pub fn write_file(path: &Path, data: &[u8], _mode: u32) -> Result<()> {
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

        let mut f = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .mode(_mode)
            .open(path)
            .map_err(|e| Error::FileError {
                path: path.to_path_buf(),
                source: e,
            })?;
        f.write_all(data).map_err(|e| Error::FileError {
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
        .map_err(|e| Error::Io(e))?;

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
