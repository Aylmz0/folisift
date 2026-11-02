# Third-party notices

## D2

Folisift invokes the unmodified D2 v0.9.0 binary for authoring-time diagram
rendering. D2 is licensed under the Mozilla Public License 2.0 (MPL-2.0).

- Project: <https://github.com/d2lang/d2>
- Source for the pinned release: <https://github.com/d2lang/d2/releases/tag/v0.9.0>
- License: <https://github.com/d2lang/d2/blob/master/LICENSE.txt>

D2 is not linked into the Rust binary and is not required to build a guide when
the generated SVG assets are present. The binary is distributed unmodified.

## PDFium

Folisift ships `libpdfium.so` as a sidecar for PDF text extraction and page
rendering. PDFium is licensed under the BSD 3-Clause licence.

- Project: <https://pdfium.googlesource.com/pdfium/>
- License: `PDFIUM-LICENSE.txt` (BSD-3-Clause)

The sidecar is loaded from the binary directory at runtime; it is not linked
into the Rust binary.

## Model2Vec

Folisift ships a quantized `potion-multilingual-128M` static embedding model
for the optional `links` helper. The model is loaded from the sidecar model
directory at runtime; it is not linked into the Rust binary.

- Project: <https://github.com/MinishLab/model2vec>
- Model: <https://huggingface.co/minishlab/potion-multilingual-128M>
- License: MIT
- Runtime crate: `model2vec-rs` (MIT), <https://github.com/MinishLab/model2vec-rs>

The model is used only for local semantic similarity; it is not a generative
model and makes no network calls after installation.

## Embedded fonts

The renderer embeds Noto Serif, Inter and Noto Sans Mono. The Noto families are
Copyright Google LLC; Inter is Copyright the Inter Project Authors. All three
are distributed under the SIL Open Font License 1.1.

- License: `templates/fonts/OFL.txt`
- Font manifest and weights: `templates/fonts/README.md`

Do not replace these font files with a different licence without updating this
notice and the packaged archive.
