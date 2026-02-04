use miette::Diagnostic;
use thiserror::Error;

/// Main error type for px operations
#[derive(Error, Diagnostic, Debug)]
pub enum PxError {
    #[error("IO error: {0}")]
    #[diagnostic(code(px::io))]
    IoError(#[from] std::io::Error),

    #[error("IO error with {path}: {message}")]
    #[diagnostic(code(px::io))]
    Io {
        path: std::path::PathBuf,
        message: String,
    },

    #[error("Parse error: {message}")]
    #[diagnostic(code(px::parse))]
    Parse {
        message: String,
        #[help]
        help: Option<String>,
    },

    #[error("Validation error: {message}")]
    #[diagnostic(code(px::validate))]
    Validation {
        message: String,
        #[help]
        help: Option<String>,
    },

    #[error("Build error: {message}")]
    #[diagnostic(code(px::build))]
    Build {
        message: String,
        #[help]
        help: Option<String>,
    },
}

pub type Result<T> = std::result::Result<T, PxError>;
