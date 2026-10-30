# Glossary

Canonical domain language for Folisift. This file defines terms; it does not
contain implementation details.

## Guide

A self-contained course/material bundle under `guides/<slug>/`: content,
configuration, assets and generated data.

## Source

An input PDF supplied by the user. Sources are read-only and never committed.

## Coverage

The auditable relationship between a source's canonical sections and the
guide content. Coverage is a claim about source topics, not a page count.

## Visual audit

A manifest with one decision for every low-text source page: redraw, table,
text, decorative or unreadable. It records what was inspected and why.

## Genre

The purpose and reader task of a document: long-form reader, revision sheet,
knowledge organiser, guided notes, workbook, reference sheet, academic paper,
brief, clinical/legal note. A genre is not a font size.

## Layout

The engine-owned visual and structural template: `paper`, `clean`, `revision`.
A layout owns geometry and component shapes.

## Structure

The required content sequence for a genre: A–G, revision-sheet,
knowledge-organiser, IMRAD, IRAC, SOAP, etc. A structure is validated by
headings, not by typography.

## Style

The complete selection of layout, palette, font pairing, density and
accessibility modifier. “Style” never means “genre”.

## Palette

A contrast-tested set of semantic colour roles (paper, ink, muted, faint,
accent, secondary, rule, code background). A palette is independent of genre.

## Font pairing

A body/heading/mono family set with verified licence, embedding and Turkish
glyph coverage. A pairing is not a palette.

## Density

The spacing rhythm of a layout: compact, comfortable or spacious. It changes
measure and leading, not the genre.

## Accessibility modifier

A change applied to any genre: standard, large-print or high-contrast.
`accessible` is a modifier, not a genre.

## Revision sheet

A short exam-preparation genre: map → must-know → comparison → worked example
→ traps → recall/answer key → one-page synthesis → review plan. It uses
micro-prose, not long textbook paragraphs.

## Knowledge organiser

A one-page-per-unit reference genre: big questions, key terms, chronology,
one map/diagram and critical distinctions. It is not a shortened revision
sheet.

## Diagram spec

The typed JSON contract that describes a diagram. The host writes the spec;
Folisift writes D2 and sanitises the SVG.

## Diagram kind

The semantic class of a diagram: concept, flow, tree, sequence, class, ERD.
The kind determines validation and layout rules.

## Evidence-informed

Designed around research and accessibility standards, without claiming that a
particular PDF is proven to raise exam scores.

## Proven style

A style whose **learning technique** or **accessibility standard** has
evidence. It is not a claim that a palette, font or decorative theme raises
scores.
