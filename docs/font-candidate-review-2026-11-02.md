# Font candidate review — 2026-11-02

- **Status:** Decision record; no new pairing added
- **Scope:** `academic` (Source Serif 4 / Source Sans 3 / Source Code Pro),
  `accessible` (Atkinson Hyperlegible Next), `plex` (IBM Plex)
- **Decision:** keep the embedded editorial/technical trio for v0.11
- **Claim policy:** font metrics and licence checks are not learning-outcome
  claims. A candidate must beat the embedded trio on coverage, package weight
  and verified Turkish glyphs before it becomes selectable.

## Method

1. Download the current variable font from the Google Fonts repository.
2. Inspect family/style names, `unitsPerEm`, `OS/2.sxHeight`, axes and cmap with
   FontTools.
3. Test Turkish glyphs: `Ç ç Ğ ğ İ ı Ö ö Ş ş Ü ü Â â Î î Û û`.
4. Read the upstream OFL notice.
5. Compare the package cost with the already embedded static families.

Fonts were inspected in `/var/tmp`; they are **not** committed. This review is
a gate, not an asset import.

## Evidence

| Candidate file | Family | Licence | Turkish glyphs | x-height / em | Axes | File size |
|---|---|---|---|---:|---|---:|
| `SourceSerif4[opsz,wght].ttf` | Source Serif 4 | OFL 1.1 | all present | 0.475 | `wght` 200–900, `opsz` 8–60 | 1.18 MB |
| `SourceSans3[wght].ttf` | Source Sans 3 | OFL 1.1 | all present | 0.478 | `wght` 200–900 | 0.62 MB |
| `SourceCodePro[wght].ttf` | Source Code Pro | OFL 1.1 | all present | 0.478 | `wght` 200–900 | 0.20 MB |
| `AtkinsonHyperlegibleNext[wght].ttf` | Atkinson Hyperlegible Next | OFL 1.1 | all present | 0.496 | `wght` 200–800 | 0.11 MB |
| `IBMPlexSerif-Regular.ttf` | IBM Plex Serif | OFL 1.1 | all present | 0.516 | static | 0.16 MB |

SHA-256 of the inspected files:

```text
97b2d4da6e3cb494b5a1e66ae176914d852ccabef49e0c02c0df25f3e39aca0b  SourceSerif4[opsz,wght].ttf
042fe2cc0b933e328410d7acbd0aa6a1873dca5aef81875f4bc214b08825c7b9  SourceSans3[wght].ttf
b400fc584e10aff25d0e775ce181b4fc1c5ea1b5dc37b81aeb2084375b945790  SourceCodePro[wght].ttf
5a455d1cfa099b601ab70751bb9673e8fe1854dc4500c80e1a220d0d75e31745  AtkinsonHyperlegibleNext[wght].ttf
e882efa9c41949a528ac2369079ec5ef050c1c996bbd0bacce3c3326d44cf80d  IBMPlexSerif-Regular.ttf
```

Embedded trio for comparison:

| Family | Role | Turkish glyphs | x-height / em | Files |
|---|---|---|---:|---|
| Noto Serif | body/serif | all present | 0.536 | regular, bold, italic, bold italic |
| Inter | heading/sans | all present | 0.546 | regular, medium, semibold, bold, italic, bold italic |
| Noto Sans Mono | code | all present | 0.536 | regular |

The embedded families already cover every Turkish glyph and ship static
regular/bold/italic faces. Their x-height is slightly larger than the Source
and Atkinson candidates at the same nominal size, which supports the current
body sizes.

## Decision

Do **not** add a candidate pairing yet.

- **Coverage:** no candidate closes a missing glyph gap; the embedded trio
  already covers Turkish.
- **Package weight:** adding Source Serif 4 + Source Sans 3 + Source Code Pro
  with static regular/bold/italic faces costs several megabytes for no verified
  readability gain. The variable files alone are 2.0 MB and do not provide the
  italic faces Typst needs for the editorial layout.
- **Atkinson:** excellent large-print candidate, but the current `accessible`
  preset already reaches a 12 pt body with the shared scale and high-contrast
  palette. A second accessible family needs a measured reading study, not a
  font swap.
- **IBM Plex:** OFL, static and Turkish-capable, but it duplicates the
  existing editorial/technical split; no measured layout problem requires it.
- **Maintenance:** each family needs licence notices, package inclusion,
  subsetting checks and style-pair validation. That cost is not justified by
  the current evidence.

## Re-open conditions

Add a candidate only when all of these hold:

1. A measured layout or accessibility problem cannot be fixed with the
   embedded trio.
2. The candidate has static regular/bold/italic faces, not only a variable
   font.
3. Turkish glyphs pass a rendered PDF check, not only a cmap check.
4. Package growth and licence notices are reviewed in the release archive.
5. A style preview and PDF baseline prove the candidate at the intended size.

Until then, `academic`, `accessible` and `plex` remain documented candidates,
not selectable ids.

## Sources

- Adobe. *Source Serif 4*, *Source Sans 3*, *Source Code Pro* (SIL OFL 1.1).
  <https://github.com/adobe-fonts>
- Google Fonts. *Atkinson Hyperlegible Next* (SIL OFL 1.1).
  <https://github.com/googlefonts/atkinson-hyperlegible-next>
- IBM. *IBM Plex* (SIL OFL 1.1). <https://github.com/ibm/plex>
- FontTools. <https://fonttools.readthedocs.io/>
