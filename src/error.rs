use std::io;
use std::path::PathBuf;

/// All errors that can occur in kx-service.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    #[error("invalid service label: {0}")]
    InvalidLabel(String),

    #[error("command `{command}` failed (exit code {code}): {message}")]
    CommandFailed {
        command: String,
        code: i32,
        message: String,
    },

    #[error("service manager not available: {0}")]
    NotAvailable(String),

    #[error("unsupported operation: {0}")]
    Unsupported(String),

    #[error("file operation failed on {path}: {source}")]
    FileError { path: PathBuf, source: io::Error },

    #[error("no native service manager found for this platform")]
    NoNativeManager,
}

pub type Result<T> = std::result::Result<T, Error>;
