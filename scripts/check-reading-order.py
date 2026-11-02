#!/usr/bin/env python3
"""Development-only tagged-PDF reading-order preflight.

Requires PyMuPDF (fitz). The product does not depend on it.

This checks the structural contract a screen reader consumes: a tagged
catalog, a language tag, a structure tree whose page order is monotonic,
heading and table roles, alternative text for content figures, and an outline
that also moves forward. It cannot replace listening to the document in
Orca/NVDA; that remains a manual release smoke step.
"""
from __future__ import annotations

import re
import sys
from pathlib import Path

import fitz

HEADING_ROLES = {"H1", "H2", "H3"}
REQUIRED_ROLES = {"P"}


def xref(value: str) -> int | None:
    match = re.search(r"(\d+) 0 R", value)
    return int(match.group(1)) if match else None


def children(doc: fitz.Document, xref_number: int) -> list[int]:
    kind, value = doc.xref_get_key(xref_number, "K")
    if kind == "xref":
        found = xref(value)
        return [found] if found is not None else []
    if kind == "array":
        return [int(number) for number in re.findall(r"(\d+) 0 R", value)]
    return []


def structure(doc: fitz.Document, root: int) -> list[tuple[str, int | None, str | None]]:
    found: list[tuple[str, int | None, str | None]] = []
    seen: set[int] = set()

    def walk(number: int) -> None:
        if number in seen:
            return
        seen.add(number)
        kind, value = doc.xref_get_key(number, "S")
        role = value.lstrip("/") if kind == "name" else "?"
        page_kind, page_value = doc.xref_get_key(number, "Pg")
        page = xref(page_value) if page_kind == "xref" else None
        alt_kind, alt_value = doc.xref_get_key(number, "Alt")
        alt = alt_value if alt_kind == "string" else None
        found.append((role, page, alt))
        for child in children(doc, number):
            walk(child)

    walk(root)
    return found


def page_number(doc: fitz.Document, page_xref: int | None) -> int | None:
    if page_xref is None:
        return None
    for index, page in enumerate(doc):
        if page.xref == page_xref:
            return index + 1
    return None


def check(path: Path) -> int:
    doc = fitz.open(path)
    errors: list[str] = []
    catalog = doc.pdf_catalog()
    if "true" not in doc.xref_get_key(catalog, "MarkInfo")[1]:
        errors.append("catalog is not marked")
    if xref(doc.xref_get_key(catalog, "StructTreeRoot")[1]) is None:
        errors.append("missing StructTreeRoot")
    if not doc.xref_get_key(catalog, "Lang")[1].strip():
        errors.append("missing document language")
    root = xref(doc.xref_get_key(catalog, "StructTreeRoot")[1])
    if root is None:
        print(f"{path}: {', '.join(errors)}", file=sys.stderr)
        return 1

    elements = structure(doc, root)
    roles = {role for role, _, _ in elements}
    missing = REQUIRED_ROLES - roles
    if missing:
        errors.append(f"missing roles {sorted(missing)}")
    if "Table" in roles and not {"TR", "TD"} <= roles:
        errors.append("table structure lacks TR/TD roles")
    if "Table" in roles and "TH" not in roles:
        errors.append("table structure lacks header cells")
    pages = [page_number(doc, page) for _, page, _ in elements if page is not None]
    if pages != sorted(pages):
        errors.append("structure tree jumps backwards across pages")
    heading_pages = [
        page_number(doc, page)
        for role, page, _ in elements
        if role in HEADING_ROLES and page is not None
    ]
    if not heading_pages:
        errors.append("no headings in structure tree")
    elif heading_pages != sorted(heading_pages):
        errors.append("heading structure order jumps backwards")

    figures = [(page, alt) for role, page, alt in elements if role == "Figure"]
    for page, alt in figures:
        number = page_number(doc, page)
        if not alt and number != 1:
            errors.append(f"content figure on page {number} has no alt text")

    parent_tree = doc.xref_get_key(catalog, "StructTreeRoot")
    root_number = xref(parent_tree[1])
    if root_number is not None:
        if doc.xref_get_key(root_number, "ParentTree")[0] == "null":
            errors.append("missing ParentTree")
    without_parents = [
        index + 1
        for index, page in enumerate(doc)
        if doc.xref_get_key(page.xref, "StructParents")[0] == "null"
    ]
    if without_parents:
        errors.append(f"pages without StructParents: {without_parents[:5]}")

    for index, page in enumerate(doc):
        sequence = [span["seqno"] for span in page.get_texttrace()]
        if sequence != sorted(sequence) or len(sequence) != len(set(sequence)):
            errors.append(f"page {index + 1} content order is not monotonic")

    outline = doc.get_toc(simple=True)
    if len(outline) < 3:
        errors.append("outline has fewer than 3 entries")
    outline_pages = [page for _, _, page in outline]
    if outline_pages != sorted(outline_pages):
        errors.append("outline jumps backwards across pages")

    if errors:
        print(f"{path}: {'; '.join(errors)}", file=sys.stderr)
        return 1
    print(
        f"{path}: {doc.page_count} pages, {len(elements)} structure elements, "
        f"{len(heading_pages)} headings, {len(figures)} figures, "
        f"{len(outline)} outline entries, order monotonic"
    )
    return 0


def main() -> int:
    if len(sys.argv) < 2:
        print("usage: check-reading-order.py <pdf>...", file=sys.stderr)
        return 2
    return max(check(Path(path)) for path in sys.argv[1:])


if __name__ == "__main__":
    raise SystemExit(main())
