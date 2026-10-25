use std::fmt::Write as _;
use std::path::Path;

use chrono::{SecondsFormat, Utc};
use serde::{Deserialize, Serialize};

use crate::error::{Result, io};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Inventory {
    pub generated_at: String,
    pub low_text_threshold: usize,
    pub sources: Vec<SourceInventory>,
    pub totals: Totals,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SourceInventory {
    pub path: String,
    pub filename: String,
    pub title: String,
    pub pages: usize,
    pub chars: usize,
    pub words: usize,
    pub low_text_threshold: usize,
    pub low_text_pages: Vec<usize>,
    pub headings: Vec<Heading>,
    pub page_map: Vec<PageRow>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Heading {
    pub heading: String,
    pub page: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PageRow {
    pub page: usize,
    pub first_line: String,
    pub chars: usize,
    pub low_text: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Totals {
    pub pages: usize,
    pub words: usize,
    pub low_text_pages: usize,
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
        if let Some(parent) = json_path.parent() {
            std::fs::create_dir_all(parent).map_err(|error| io(parent, error))?;
        }
        if let Some(parent) = markdown_path.parent() {
            std::fs::create_dir_all(parent).map_err(|error| io(parent, error))?;
        }

        let json =
            serde_json::to_string_pretty(self).map_err(|error| crate::error::Error::Parse {
                path: json_path.to_path_buf(),
                message: error.to_string(),
            })?;
        std::fs::write(json_path, json).map_err(|error| io(json_path, error))?;
        std::fs::write(markdown_path, self.to_markdown())
            .map_err(|error| io(markdown_path, error))?;
        Ok(())
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
            let _ = writeln!(out, "- Düşük metinli sayfalar: **{low_pages}**\n");
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
            low_text_pages: vec![1],
            headings: vec![Heading {
                heading: "0.1 Giriş".to_string(),
                page: 2,
            }],
            page_map: vec![
                PageRow {
                    page: 1,
                    first_line: "Bölüm: 0".to_string(),
                    chars: 8,
                    low_text: true,
                },
                PageRow {
                    page: 2,
                    first_line: "0.1 Giriş".to_string(),
                    chars: 122,
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
}
