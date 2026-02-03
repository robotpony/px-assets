use miette::Diagnostic;
use thiserror::Error;

/// Main error type for px operations
#[derive(Error, Diagnostic, Debug)]
pub enum PxError {
    #[error("IO error: {0}")]
    #[diagnostic(code(px::io))]
    Io(#[from] std::io::Error),

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
