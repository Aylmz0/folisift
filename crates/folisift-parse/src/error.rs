use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("PDFium is not usable: {message}")]
    Pdfium { message: String },

    #[error("could not parse {path}: {message}")]
    Parse { path: PathBuf, message: String },

    #[error("I/O error at {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("invalid page selection: {value}")]
    PageSelection { value: String },

    #[error("PDF has no pages: {path}")]
    NoPages { path: PathBuf },
}

pub type Result<T> = std::result::Result<T, Error>;

pub(crate) fn io(path: impl Into<PathBuf>, source: std::io::Error) -> Error {
    Error::Io {
        path: path.into(),
        source,
    }
}
