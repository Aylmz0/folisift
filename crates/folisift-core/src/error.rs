use std::path::PathBuf;

/// Errors returned by the core crate.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("I/O error at {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("invalid JSON in {path}: {source}")]
    Json {
        path: PathBuf,
        #[source]
        source: serde_json::Error,
    },

    #[error("invalid guide config {path}: {message}")]
    Config { path: PathBuf, message: String },

    #[error("path escapes the workspace: {path}")]
    PathEscape { path: PathBuf },

    #[error("writes are not allowed under {path}")]
    ReadOnly { path: PathBuf },

    #[error("guide not found: {slug}")]
    GuideNotFound { slug: String },

    #[error("invalid slug: {slug}")]
    InvalidSlug { slug: String },

    #[error("invalid target page range: {range}")]
    InvalidTargetPages { range: String },

    #[error("unsupported guide schema version: {version}")]
    UnsupportedSchema { version: u32 },
}

pub type Result<T> = std::result::Result<T, Error>;

pub(crate) fn io(path: impl Into<PathBuf>, source: std::io::Error) -> Error {
    Error::Io {
        path: path.into(),
        source,
    }
}
