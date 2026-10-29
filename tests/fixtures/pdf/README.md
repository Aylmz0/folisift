# Synthetic PDF fixtures

These tiny PDFs exist only to prove the negative audit paths. They are not
content examples. `generate.py` regenerates them byte-for-byte:

```bash
python3 tests/fixtures/pdf/generate.py
```

| File | Seeded defect | Expected field |
|---|---|---|
| `orphan-heading.pdf` | heading at the bottom with no body below | `orphan_heading_pages = [1]` |
| `box-overflow.pdf` | text starts outside the content box | `content_box_pages = [1]` |
| `image-overflow.pdf` | image extends past the page edge | `image_overflow_pages = [1]` |
| `bookmark-mismatch.pdf` | bookmark title is absent from its page | `bookmark_mismatches.len() = 1` |

`crates/folisift-parse/tests/audit.rs` asserts each expected field and that the
overall audit fails. The fixtures are deliberately untagged and have no useful
bookmarks; the tests target one defect at a time rather than treating the whole
audit result as a boolean.
