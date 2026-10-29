# Structure packs

A structure pack names the sections a guide must contain. The engine validates
the headings after parsing frontmatter and before Typst compilation. A missing
section is a build error, not a warning.

Packs live in `structures/<id>.json`:

```json
{
  "id": "imrad",
  "title": "IMRAD research report",
  "sections": [
    {
      "id": "introduction",
      "tr": "Giriş",
      "en": "Introduction",
      "hint": "Araştırma sorusu, bağlam ve boşluk."
    }
  ]
}
```

The heading matcher ignores numbering, case, punctuation and Turkish
dotted/dotless `i`, so `## 1. Giriş`, `## GİRİŞ` and `## Giris` are equivalent.

## Built-in packs

| Id | Required sections |
|---|---|
| `a-g` | none; the course-shaped A–G layout |
| `imrad` | Introduction, Methods, Results, Discussion |
| `irac` | Issue, Rule, Application, Conclusion |
| `soap` | Subjective, Objective, Assessment, Plan |
| `formula-sheet` | Symbols, Formulas, Units, Pitfalls |
| `literature-matrix` | Research question, Sources, Synthesis, Gaps |
| `lecture-notes` | Learning goals, Key concepts, Examples, Summary |
| `executive-brief` | Summary, Key findings, Recommendations, Risks |

`a-g` has no required sections because the coverage gate already maps every
numbered source subsection to a guide heading. The audit section is required
for every structure.

## New guide

```bash
cargo run -p folisift-cli -- scaffold \
  --slug paper-review --title "Paper review" \
  --style clean --structure imrad --language en --domain cs \
  --source sources/PD2.pdf
```

The scaffold writes one numbered heading per required section, in the selected
language. Replace the placeholder prose with source-grounded content; do not
delete required headings.

## Adding a pack

1. Add `structures/<id>.json`.
2. Add the id to `STRUCTURES` in `crates/folisift-core/src/config.rs`.
3. Add the `include_str!` entry in `crates/folisift-core/src/structures.rs`.
4. Add a fixture or a unit test that proves a missing section fails.
5. Update this table.
