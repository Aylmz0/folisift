use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use crate::error::{Error, Result};

/// Optional directory containing the PDFium shared library.
pub const PDFIUM_DIR_ENV: &str = "FOLISIFT_PDFIUM_DIR";

static INIT: OnceLock<std::result::Result<(), String>> = OnceLock::new();

/// Load PDFium once, preferring the sidecar next to the executable.
///
/// Search order:
/// 1. an explicit directory passed by the caller;
/// 2. `FOLISIFT_PDFIUM_DIR`;
/// 3. `PDFIUM_LIB_PATH`;
/// 4. the directory containing the `folisift` executable;
/// 5. liteparse's build-time cache (development fallback).
pub fn init(explicit_dir: Option<&Path>) -> Result<()> {
    match INIT.get_or_init(|| init_inner(explicit_dir)) {
        Ok(()) => Ok(()),
        Err(message) => Err(Error::Pdfium {
            message: message.clone(),
        }),
    }
}

fn init_inner(explicit_dir: Option<&Path>) -> std::result::Result<(), String> {
    let explicit = explicit_dir.map(Path::to_path_buf);
    let env_dir = std::env::var_os(PDFIUM_DIR_ENV).map(PathBuf::from);
    let loader_dir = std::env::var_os("PDFIUM_LIB_PATH").map(PathBuf::from);
    let pdfium_dir = explicit.or(env_dir).or(loader_dir).or_else(executable_dir);

    if let Some(dir) = pdfium_dir {
        let lib = dir.join(library_name());
        if lib.is_file() {
            // SAFETY: PDFium is initialised once, before parser threads start.
            unsafe { std::env::set_var("PDFIUM_LIB_PATH", &dir) };
        }
    }

    liteparse_pdfium::Library::try_init()
        .map(|_| ())
        .map_err(|error| {
            format!(
                "{error}. Place {} next to the folisift executable or set {}.",
                library_name(),
                PDFIUM_DIR_ENV
            )
        })
}

fn executable_dir() -> Option<PathBuf> {
    let exe = std::env::current_exe().ok()?;
    let dir = exe.parent()?.to_path_buf();
    let lib = dir.join(library_name());
    lib.is_file().then_some(dir)
}

fn library_name() -> &'static str {
    if cfg!(target_os = "windows") {
        "pdfium.dll"
    } else if cfg!(target_os = "macos") {
        "libpdfium.dylib"
    } else {
        "libpdfium.so"
    }
}

/// Read the PDF `/Info` title without going through the full parser.
pub fn document_title(path: &Path) -> Result<String> {
    init(None)?;
    let path = path.to_str().ok_or_else(|| Error::Pdfium {
        message: "non-UTF-8 source path".to_string(),
    })?;
    let library = liteparse_pdfium::Library::try_init().map_err(|error| Error::Pdfium {
        message: error.to_string(),
    })?;
    let document = library
        .load_document(path, None)
        .map_err(|error| Error::Parse {
            path: PathBuf::from(path),
            message: error.to_string(),
        })?;
    Ok(document.meta_text("Title").unwrap_or_default())
}
