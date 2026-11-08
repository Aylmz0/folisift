use std::fmt::Write as _;
use std::path::{Path, PathBuf};

use chrono::{SecondsFormat, Utc};
use serde::{Deserialize, Serialize};

use crate::error::{Result, io};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Inventory {
    pub generated_at: String,
    pub low_text_threshold: usize,
    pub sources: Vec<SourceInventory>,
    pub totals: Totals,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SourceInventory {
    pub path: String,
    pub filename: String,
    pub title: String,
    pub pages: usize,
    pub chars: usize,
    pub words: usize,
    pub low_text_threshold: usize,
    /// BLAKE3 of the source PDF at analysis time; absent in old inventories.
    #[serde(default)]
    pub source_digest: Option<String>,
    pub low_text_pages: Vec<usize>,
    pub headings: Vec<Heading>,
    /// Compact layout summary; full blocks live in the separate layout artifact.
    #[serde(default)]
    pub layout: Option<crate::layout::LayoutSummary>,
    /// Sections recovered from the source's own topic list/TOC.
    ///
    /// Heading extraction misses styled or wrapped headings. This list is the
    /// canonical coverage set when present; `headings` remains the page map.
    #[serde(default)]
    pub sections: Vec<CanonicalSection>,
    pub page_map: Vec<PageRow>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CanonicalSection {
    pub number: String,
    pub title: String,
    /// 0: the topic list does not give the actual section page.
    #[serde(default)]
    pub page: usize,
    /// `toc`, `toc-sequence` (repeated source numbering) or `manual`.
    #[serde(default = "default_section_origin")]
    pub origin: String,
}

fn default_section_origin() -> String {
    "toc".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Heading {
    pub heading: String,
    pub page: usize,
    /// Heading level from the layout classifier when available.
    #[serde(default)]
    pub level: Option<u8>,
    /// Stable block id from the layout artifact when available.
    #[serde(default)]
    pub block_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PageRow {
    pub page: usize,
    pub first_line: String,
    pub chars: usize,
    pub low_text: bool,
    #[serde(default)]
    pub blocks: usize,
    #[serde(default)]
    pub headings: usize,
    #[serde(default)]
    pub tables: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Totals {
    pub pages: usize,
    pub words: usize,
    pub low_text_pages: usize,
}

impl Inventory {
    /// True when every source file still matches the digest recorded at analysis
    /// time. Old inventories without digests are treated as stale.
    pub fn matches_sources(&self, paths: &[PathBuf]) -> bool {
        if self.sources.len() != paths.len() {
            return false;
        }
        self.sources.iter().all(|source| {
            let Some(digest) = &source.source_digest else {
                return false;
            };
            paths
                .iter()
                .filter(|path| {
                    path.file_name().and_then(|name| name.to_str())
                        == Some(source.filename.as_str())
                })
                .any(|path| {
                    std::fs::read(path)
                        .map(|bytes| blake3::hash(&bytes).to_hex().to_string() == *digest)
                        .unwrap_or(false)
                })
        })
    }
}

impl SourceInventory {
    pub fn title_or_filename(&self) -> &str {
        if self.title.is_empty() {
            &self.filename
        } else {
            &self.title
        }
    }
}

impl Inventory {
    /// Markdown review for ambiguous/reconciled headings. This is a reading
    /// aid, not a gate: a row means "inspect me", not "wrong".
    pub fn headings_review(&self) -> String {
        let mut out = String::from("# Heading review\n\n");
        out.push_str("| Source | Page | Heading | Level | Status |\n|---|---:|---|---:|---|\n");
        for source in &self.sources {
            let mut seen = std::collections::BTreeSet::new();
            let canonical = source
                .sections
                .iter()
                .map(|section| section.number.as_str())
                .collect::<std::collections::BTreeSet<_>>();
            for heading in &source.headings {
                let number = heading
                    .heading
                    .split_whitespace()
                    .next()
                    .unwrap_or_default()
                    .trim_end_matches('.');
                let mut status = "ok";
                if !seen.insert(number.to_string()) {
                    status = "duplicate number";
                } else if !number.contains('.') {
                    status = "non-section heading";
                } else if !canonical.is_empty() && !canonical.contains(number) {
                    status = "not in TOC";
                }
                out.push_str(&format!(
                    "| {} | {} | {} | {} | {} |\n",
                    source.filename,
                    heading.page,
                    heading.heading.replace('|', "\\|"),
                    heading.level.unwrap_or(0),
                    status
                ));
            }
            for section in &source.sections {
                if !source.headings.iter().any(|heading| {
                    heading
                        .heading
                        .split_whitespace()
                        .next()
                        .unwrap_or_default()
                        .trim_end_matches('.')
                        == section.number
                }) {
                    out.push_str(&format!(
                        "| {} | — | {} {} | — | TOC section missing in body |\n",
                        source.filename,
                        section.number,
                        section.title.replace('|', "\\|")
                    ));
                }
            }
        }
        out.push_str("\nReview rows are not a coverage claim; fix the source map or add a manual override.\n");
        out
    }

    pub fn new(low_text_threshold: usize, sources: Vec<SourceInventory>) -> Self {
        let totals = Totals {
            pages: sources.iter().map(|source| source.pages).sum(),
            words: sources.iter().map(|source| source.words).sum(),
            low_text_pages: sources
                .iter()
                .map(|source| source.low_text_pages.len())
                .sum(),
        };
        Self {
            generated_at: Utc::now().to_rfc3339_opts(SecondsFormat::Secs, false),
            low_text_threshold,
            sources,
            totals,
        }
    }

    pub fn write_files(&self, json_path: &Path, markdown_path: &Path) -> Result<()> {
        if let Some(parent) = markdown_path.parent() {
            std::fs::create_dir_all(parent).map_err(|error| io(parent, error))?;
        }
        self.write_json(json_path)?;
        std::fs::write(markdown_path, self.to_markdown()).map_err(|error| io(markdown_path, error))
    }

    /// Write the canonical JSON bytes for this inventory. The MCP server and
    /// the CLI use this so a stored inventory matches the in-memory one.
    pub fn write_json(&self, json_path: &Path) -> Result<()> {
        if let Some(parent) = json_path.parent() {
            std::fs::create_dir_all(parent).map_err(|error| io(parent, error))?;
        }
        let json =
            serde_json::to_string_pretty(self).map_err(|error| crate::error::Error::Parse {
                path: json_path.to_path_buf(),
                message: error.to_string(),
            })?;
        std::fs::write(json_path, json).map_err(|error| io(json_path, error))
    }

    pub fn to_markdown(&self) -> String {
        let mut out = String::new();
        let _ = writeln!(out, "# Kaynak Envanteri\n");
        let _ = writeln!(out, "Üretim: {}\n", self.generated_at);
        let _ = writeln!(
            out,
            "| PDF | Sayfa | Kelime | Düşük metinli sayfa |\n|---|---:|---:|---:|"
        );
        for source in &self.sources {
            let _ = writeln!(
                out,
                "| {} | {} | {} | {} |",
                source.filename,
                source.pages,
                source.words,
                source.low_text_pages.len()
            );
        }
        let _ = writeln!(
            out,
            "\n**Toplam:** {} slayt, {} kelime, {} düşük metinli slayt.\n",
            self.totals.pages, self.totals.words, self.totals.low_text_pages
        );

        for source in &self.sources {
            let title = if source.title.is_empty() {
                "Başlıksız"
            } else {
                &source.title
            };
            let low_pages = if source.low_text_pages.is_empty() {
                "yok".to_string()
            } else {
                source
                    .low_text_pages
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(", ")
            };
            let _ = writeln!(out, "## {} — {}\n", source.filename, title);
            let _ = writeln!(out, "- Sayfa: **{}**", source.pages);
            let _ = writeln!(out, "- Kelime: **{}**", source.words);
            let _ = writeln!(out, "- Düşük metinli sayfalar: **{low_pages}**");
            if let Some(layout) = &source.layout {
                let _ = writeln!(
                    out,
                    "- Yerleşim: **{}** blok, **{}** başlık, **{}** tablo, **{}** figür ({} sayfa)\n",
                    layout.blocks,
                    layout.headings,
                    layout.tables,
                    layout.figures,
                    layout.pages_with_blocks
                );
            } else {
                out.push('\n');
            }
            let _ = writeln!(out, "### Başlıklar\n");
            let _ = writeln!(out, "| Sayfa | Başlık |\n|---:|---|");
            for heading in &source.headings {
                let _ = writeln!(
                    out,
                    "| {} | {} |",
                    heading.page,
                    escape_pipes(&heading.heading)
                );
            }
            if source.headings.is_empty() {
                let _ = writeln!(out, "| — | Başlık çıkarılamadı; sayfa haritasına bakın. |");
            }
            if !source.sections.is_empty() {
                let _ = writeln!(out, "\n### Kanonik Bölümler\n");
                let _ = writeln!(out, "| No | Başlık |\n|---|---|");
                for section in &source.sections {
                    let _ = writeln!(
                        out,
                        "| {} | {} |",
                        escape_pipes(&section.number),
                        escape_pipes(&section.title)
                    );
                }
            }
            let _ = writeln!(out, "\n### Sayfa Haritası\n");
            let _ = writeln!(
                out,
                "| Sayfa | İlk anlamlı satır | Düşük metin |\n|---:|---|---|"
            );
            for row in &source.page_map {
                let flag = if row.low_text { "evet" } else { "" };
                let _ = writeln!(
                    out,
                    "| {} | {} | {} |",
                    row.page,
                    escape_pipes(&row.first_line),
                    flag
                );
            }
            out.push('\n');
        }
        format!("{}\n", out.trim_end())
    }
}

fn escape_pipes(text: &str) -> String {
    text.replace('|', "\\|")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn markdown_contains_totals_and_pages() {
        let source = SourceInventory {
            path: "/tmp/PD0.pdf".to_string(),
            filename: "PD0.pdf".to_string(),
            title: "Programlama Dilleri".to_string(),
            pages: 2,
            chars: 130,
            words: 20,
            low_text_threshold: 120,
            source_digest: None,
            low_text_pages: vec![1],
            headings: vec![Heading {
                heading: "0.1 Giriş".to_string(),
                page: 2,
                level: None,
                block_id: None,
            }],
            layout: None,
            sections: vec![],
            page_map: vec![
                PageRow {
                    page: 1,
                    first_line: "Bölüm: 0".to_string(),
                    chars: 8,
                    low_text: true,
                    blocks: 0,
                    headings: 0,
                    tables: 0,
                },
                PageRow {
                    page: 2,
                    first_line: "0.1 Giriş".to_string(),
                    chars: 122,
                    blocks: 0,
                    headings: 0,
                    tables: 0,
                    low_text: false,
                },
            ],
        };
        let inventory = Inventory::new(120, vec![source]);
        let markdown = inventory.to_markdown();
        assert!(markdown.contains("**Toplam:** 2 slayt, 20 kelime, 1 düşük metinli slayt."));
        assert!(markdown.contains("| PD0.pdf | 2 | 20 | 1 |"));
        assert!(markdown.contains("| 2 | 0.1 Giriş |"));
    }
    #[test]
    fn review_flags_toc_sections_missing_from_the_body() {
        let inventory = Inventory::new(
            120,
            vec![SourceInventory {
                path: "PD0.pdf".to_string(),
                filename: "PD0.pdf".to_string(),
                title: "Demo".to_string(),
                pages: 4,
                chars: 0,
                words: 0,
                low_text_threshold: 120,
                source_digest: None,
                low_text_pages: vec![],
                headings: vec![Heading {
                    heading: "1.1 Giriş".to_string(),
                    page: 2,
                    level: Some(2),
                    block_id: None,
                }],
                layout: None,
                sections: vec![
                    CanonicalSection {
                        number: "1.1".to_string(),
                        title: "Giriş".to_string(),
                        page: 2,
                        origin: "toc".to_string(),
                    },
                    CanonicalSection {
                        number: "1.2".to_string(),
                        title: "Eksik bölüm".to_string(),
                        page: 3,
                        origin: "toc".to_string(),
                    },
                ],
                page_map: vec![],
            }],
        );
        let review = inventory.headings_review();
        assert!(review.contains("1.1 Giriş"));
        assert!(review.contains("TOC section missing in body"));
        assert!(review.contains("1.2 Eksik bölüm"));
    }
}
