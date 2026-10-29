use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("frontmatter error: {0}")]
    Frontmatter(String),

    #[error("Markdown error at line {line}: {message}")]
    Markdown { line: usize, message: String },

    #[error("Typst error: {0}")]
    Typst(String),

    #[error("style {id:?}: {message}")]
    Style { id: String, message: String },

    #[error("style config: {0}")]
    StyleConfig(String),

    #[error("diagram: {0}")]
    Diagram(String),

    #[error("I/O error at {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
}

pub type Result<T> = std::result::Result<T, Error>;
