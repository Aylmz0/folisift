use std::path::{Path, PathBuf};
use std::sync::LazyLock;

use typst::LibraryExt;
use typst::diag::{FileError, FileResult};
use typst::foundations::{Bytes, Datetime};
use typst::syntax::{FileId, RootedPath, Source, VirtualPath, VirtualRoot};
use typst::text::{Font, FontBook};
use typst::utils::LazyHash;
use typst::{Library, World};

static FONT_DATA: [&[u8]; 11] = [
    include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../templates/fonts/NotoSerif-Regular.ttf"
    )),
    include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../templates/fonts/NotoSerif-Bold.ttf"
    )),
    include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../templates/fonts/NotoSerif-Italic.ttf"
    )),
    include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../templates/fonts/NotoSerif-BoldItalic.ttf"
    )),
    include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../templates/fonts/Inter-Regular.ttf"
    )),
    include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../templates/fonts/Inter-Medium.ttf"
    )),
    include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../templates/fonts/Inter-SemiBold.ttf"
    )),
    include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../templates/fonts/Inter-Bold.ttf"
    )),
    include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../templates/fonts/Inter-Italic.ttf"
    )),
    include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../templates/fonts/Inter-BoldItalic.ttf"
    )),
    include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../templates/fonts/NotoSansMono-Regular.ttf"
    )),
];

static FONTS: LazyLock<Vec<Font>> = LazyLock::new(|| {
    let mut fonts = Vec::new();
    for data in FONT_DATA {
        fonts.extend(Font::iter(Bytes::new(data)));
    }
    fonts
});

static BOOK: LazyLock<LazyHash<FontBook>> =
    LazyLock::new(|| LazyHash::new(FontBook::from_fonts(FONTS.iter())));

static LIBRARY: LazyLock<LazyHash<Library>> =
    LazyLock::new(|| LazyHash::new(Library::builder().build()));

pub struct TypstWorld {
    root: PathBuf,
    main: FileId,
    source: Source,
}

impl TypstWorld {
    pub fn new(root: &Path, source_text: String) -> Result<Self, String> {
        let root = root
            .canonicalize()
            .map_err(|error| format!("invalid guide root {}: {error}", root.display()))?;
        let path = VirtualPath::new("main.typ")
            .map_err(|error| format!("invalid Typst main path: {error}"))?;
        let main = FileId::new(RootedPath::new(VirtualRoot::Project, path));
        Ok(Self {
            root,
            main,
            source: Source::new(main, source_text),
        })
    }
}

impl World for TypstWorld {
    fn library(&self) -> &LazyHash<Library> {
        &LIBRARY
    }

    fn book(&self) -> &LazyHash<FontBook> {
        &BOOK
    }

    fn main(&self) -> FileId {
        self.main
    }

    fn source(&self, id: FileId) -> FileResult<Source> {
        if id == self.main {
            return Ok(self.source.clone());
        }
        let bytes = self.file(id)?;
        let text = std::str::from_utf8(bytes.as_slice())
            .map_err(|_| FileError::InvalidUtf8)?
            .to_string();
        Ok(Source::new(id, text))
    }

    fn file(&self, id: FileId) -> FileResult<Bytes> {
        let path = id.vpath().realize(&self.root).map_err(FileError::Realize)?;
        if !path.starts_with(&self.root) {
            return Err(FileError::AccessDenied);
        }
        let bytes = std::fs::read(&path).map_err(|error| FileError::from_io(error, &path))?;
        Ok(Bytes::new(bytes))
    }

    fn font(&self, index: usize) -> Option<Font> {
        FONTS.get(index).cloned()
    }

    fn today(&self, _offset: Option<typst::foundations::Duration>) -> Option<Datetime> {
        Datetime::from_ymd(2026, 10, 26)
    }
}
