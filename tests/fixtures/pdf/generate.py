#!/usr/bin/env python3
"""Generate tiny synthetic PDFs used by the negative audit tests.

The files are checked in. Keeping the generator next to them makes the
intent reproducible without pulling a PDF library into the test suite.
"""

from pathlib import Path

HERE = Path(__file__).parent
WIDTH, HEIGHT = 595, 842


def build(objects: list[bytes]) -> bytes:
    out = bytearray(b"%PDF-1.4\n%\xe2\xe3\xcf\xd3\n")
    offsets = [0]
    for number, body in enumerate(objects, start=1):
        offsets.append(len(out))
        out.extend(f"{number} 0 obj\n".encode())
        out.extend(body)
        out.extend(b"\nendobj\n")
    xref = len(out)
    out.extend(f"xref\n0 {len(objects) + 1}\n".encode())
    out.extend(b"0000000000 65535 f \n")
    for offset in offsets[1:]:
        out.extend(f"{offset:010d} 00000 n \n".encode())
    out.extend(
        f"trailer\n<< /Size {len(objects) + 1} /Root 1 0 R >>\nstartxref\n{xref}\n%%EOF\n".encode()
    )
    return bytes(out)


def stream(content: bytes) -> bytes:
    return b"<< /Length " + str(len(content)).encode() + b" >>\nstream\n" + content + b"\nendstream"


def catalog(pages: int = 2, outlines: int = 0) -> bytes:
    extra = f" /Outlines {outlines} 0 R /PageMode /UseOutlines".encode() if outlines else b""
    return b"<< /Type /Catalog /Pages " + str(pages).encode() + b" 0 R" + extra + b" >>"


def pages(page: int) -> bytes:
    return f"<< /Type /Pages /Kids [{page} 0 R] /Count 1 >>".encode()


def page(contents: int, resources: bytes) -> bytes:
    return (
        f"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 {WIDTH} {HEIGHT}] "
        f"/Resources {resources.decode()} /Contents {contents} 0 R >>"
    ).encode()


def font() -> bytes:
    return b"<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>"


def simple_pdf(content: bytes) -> bytes:
    resources = b"<< /Font << /F1 5 0 R >> >>"
    return build([catalog(), pages(3), page(4, resources), stream(content), font()])


def orphan_heading() -> bytes:
    return simple_pdf(b"BT /F1 14 Tf 72 60 Td (1.1 Orphan heading) Tj ET")


def box_overflow() -> bytes:
    return simple_pdf(b"BT /F1 12 Tf 560 400 Td (Outside the content box) Tj ET")


def image_overflow() -> bytes:
    # 1x1 red image scaled to 100x100 at x=560; right edge is 660 > 595.
    content = b"q 100 0 0 100 560 700 cm /Im0 Do Q"
    resources = b"<< /XObject << /Im0 5 0 R >> >>"
    image = (
        b"<< /Type /XObject /Subtype /Image /Width 1 /Height 1 "
        b"/ColorSpace /DeviceRGB /BitsPerComponent 8 /Length 3 >>\n"
        b"stream\n\xff\x00\x00\nendstream"
    )
    return build([catalog(), pages(3), page(4, resources), stream(content), image])


def bookmark_mismatch() -> bytes:
    content = b"BT /F1 12 Tf 72 700 Td (Actual heading) Tj ET"
    resources = b"<< /Font << /F1 5 0 R >> >>"
    outline_root = b"<< /Type /Outlines /First 7 0 R /Last 7 0 R /Count 1 >>"
    outline_item = (
        b"<< /Title (Wrong destination) /Parent 6 0 R "
        b"/Dest [3 0 R /XYZ 0 800 null] >>"
    )
    return build(
        [
            catalog(outlines=6),
            pages(3),
            page(4, resources),
            stream(content),
            font(),
            outline_root,
            outline_item,
        ]
    )


def main() -> None:
    files = {
        "orphan-heading.pdf": orphan_heading(),
        "box-overflow.pdf": box_overflow(),
        "image-overflow.pdf": image_overflow(),
        "bookmark-mismatch.pdf": bookmark_mismatch(),
    }
    for name, data in files.items():
        (HERE / name).write_bytes(data)
        print(f"{name}: {len(data)} bytes")


if __name__ == "__main__":
    main()
