use std::fmt::Write as _;
use std::path::Path;
use std::sync::LazyLock;

use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::error::{Result, io};
use crate::inventory::Inventory;

pub const DEFAULT_THRESHOLD: f64 = 0.55;

static SECTION_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(\d+\.\d+)\s+(.+)$").expect("valid section regex"));

pub const MIN_SECTION_CHARS: usize = 400;
pub const WEAK_SECTION_CHARS: usize = 600;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverageReport {
    pub passed: bool,
    pub threshold: f64,
    pub total: usize,
    pub missing: usize,
    pub empty_sections: Vec<String>,
    pub weak_sections: Vec<String>,
    pub unknown_claims: Vec<String>,
    pub sources: Vec<SourceCoverage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceCoverage {
    pub filename: String,
    pub pages: usize,
    pub low_text_pages: usize,
    pub sections: Vec<SectionResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SectionResult {
    pub number: String,
    pub title: String,
    pub page: usize,
    pub score: f64,
    pub matched: String,
    pub body_chars: usize,
    pub empty: bool,
    pub ok: bool,
}

/// Compare every numbered source subsection with the guide's Markdown lines.
pub fn check_coverage(
    inventory: &Inventory,
    guide_markdown: &str,
    threshold: f64,
) -> CoverageReport {
    let lines = guide_lines(guide_markdown);
    let sections_by_number = guide_section_chars(guide_markdown);
    let claim_chars = guide_claim_chars(guide_markdown);
    let mut known_claims = std::collections::HashSet::new();
    for source in &inventory.sources {
        let id = source.filename.trim_end_matches(".pdf");
        for heading in &source.headings {
            if let Some(captures) = SECTION_RE.captures(heading.heading.trim()) {
                known_claims.insert(format!("{id}:{}", &captures[1]));
            }
        }
    }
    let unknown_claims = claim_chars
        .keys()
        .filter(|key| !known_claims.contains(*key))
        .cloned()
        .collect::<Vec<_>>();
    let mut sources = Vec::with_capacity(inventory.sources.len());
    let mut total = 0;
    let mut missing = 0;
    let mut empty_sections = Vec::new();
    let mut weak_sections = Vec::new();

    for source in &inventory.sources {
        let mut sections = Vec::new();
        let mut seen = std::collections::HashSet::new();
        for heading in &source.headings {
            let Some(captures) = SECTION_RE.captures(heading.heading.trim()) else {
                continue;
            };
            let number = captures
                .get(1)
                .map_or("", |value| value.as_str())
                .to_string();
            if !seen.insert(number.clone()) {
                continue;
            }
            let title = captures
                .get(2)
                .map_or("", |value| value.as_str())
                .trim()
                .to_string();
            total += 1;
            let source_id = source.filename.trim_end_matches(".pdf");
            let claim_key = format!("{source_id}:{number}");
            let claimed = claim_chars.get(&claim_key).copied();
            let (score, matched) = if claimed.is_some() {
                (1.0, format!("<!-- covers: {claim_key} -->"))
            } else {
                best_line(&number, &title, &lines)
            };
            let heading_ok = claimed.is_some() || score >= threshold;
            if !heading_ok {
                missing += 1;
            }
            let body_chars = claimed
                .or_else(|| sections_by_number.get(&number).copied())
                .unwrap_or(0);
            let empty = body_chars < MIN_SECTION_CHARS;
            if empty {
                empty_sections.push(format!("{number} {title}"));
            } else if body_chars < WEAK_SECTION_CHARS {
                weak_sections.push(format!("{number} {title}"));
            }
            sections.push(SectionResult {
                number,
                title,
                page: heading.page,
                score,
                matched,
                body_chars,
                empty,
                ok: heading_ok && !empty,
            });
        }
        sources.push(SourceCoverage {
            filename: source.filename.clone(),
            pages: source.pages,
            low_text_pages: source.low_text_pages.len(),
            sections,
        });
    }

    CoverageReport {
        passed: missing == 0 && empty_sections.is_empty() && unknown_claims.is_empty(),
        threshold,
        total,
        missing,
        empty_sections,
        weak_sections,
        unknown_claims,
        sources,
    }
}

impl CoverageReport {
    pub fn write_markdown(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|error| io(parent, error))?;
        }
        std::fs::write(path, self.to_markdown()).map_err(|error| io(path, error))
    }

    pub fn to_markdown(&self) -> String {
        let mut out = String::new();
        let _ = writeln!(out, "# Kapsam Denetimi\n");
        let _ = writeln!(out, "Eşik: **{:.2}**\n", self.threshold);
        for source in &self.sources {
            let _ = writeln!(out, "## {} — {} slayt\n", source.filename, source.pages);
            let _ = writeln!(
                out,
                "| Bölüm | Kaynak sayfa | Durum | Gövde karakteri | Eşleşen kılavuz satırı |\n|---|---:|---|---:|---|"
            );
            for section in &source.sections {
                let status = if section.empty {
                    "BOŞ"
                } else if section.ok {
                    "OK"
                } else {
                    "EKSİK"
                };
                let matched = section.matched.replace('|', "\\|");
                let matched = matched.chars().take(150).collect::<String>();
                let _ = writeln!(
                    out,
                    "| {} {} | {} | {} | {} | {} |",
                    section.number,
                    section.title,
                    section.page,
                    status,
                    section.body_chars,
                    matched
                );
            }
            let _ = writeln!(
                out,
                "\nAlt bölüm: **{}** · Düşük metinli slayt: **{}**\n",
                source.sections.len(),
                source.low_text_pages
            );
        }
        let _ = writeln!(out, "## Sonuç\n");
        let _ = writeln!(out, "- Denetlenen alt bölüm: **{}**", self.total);
        let _ = writeln!(out, "- Eksik/weak match: **{}**", self.missing);
        let _ = writeln!(out, "- Boş bölüm: **{}**", self.empty_sections.len());
        let _ = writeln!(
            out,
            "- Bilinmeyen covers: **{}**",
            self.unknown_claims.len()
        );
        let _ = writeln!(out, "- Zayıf bölüm: **{}**", self.weak_sections.len());
        let _ = writeln!(
            out,
            "- Sonuç: **{}**\n",
            if self.passed {
                "GEÇTİ"
            } else {
                "KONTROL GEREKLİ"
            }
        );
        let _ = writeln!(
            out,
            "Not: Bu otomatik denetim başlık düzeyindedir. Tablo, şekil, formül ve örnek kapsamı ayrıca G bölümünde elle doğrulanmalıdır."
        );
        out
    }
}

/// Body character counts for every `## <number> <title>` guide section.
pub fn guide_body_chars(markdown: &str) -> std::collections::HashMap<String, usize> {
    guide_section_chars(markdown)
}

fn guide_claim_chars(markdown: &str) -> std::collections::HashMap<String, usize> {
    let markdown = strip_frontmatter(markdown);
    let mut claims = std::collections::HashMap::new();
    let mut pending: Vec<String> = Vec::new();
    let mut current: Option<(Vec<String>, String)> = None;
    let mut fenced = false;

    let finalize = |claims: &mut std::collections::HashMap<String, usize>,
                    current: Option<(Vec<String>, String)>| {
        if let Some((keys, body)) = current {
            for key in keys {
                claims.insert(key, count_body_chars(&body));
            }
        }
    };

    for line in markdown.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
            fenced = !fenced;
            continue;
        }
        if fenced {
            continue;
        }
        if let Some(keys) = covers_comment(trimmed) {
            pending.extend(keys);
            continue;
        }
        if trimmed.starts_with('#') {
            finalize(&mut claims, current.take());
            if !pending.is_empty() {
                current = Some((std::mem::take(&mut pending), String::new()));
            }
            continue;
        }
        if let Some((_, body)) = &mut current {
            body.push_str(trimmed);
            body.push('\n');
        }
    }
    finalize(&mut claims, current);
    claims
}

fn covers_comment(line: &str) -> Option<Vec<String>> {
    let rest = line.strip_prefix("<!--")?.strip_suffix("-->")?.trim();
    let list = rest.strip_prefix("covers:")?.trim();
    let keys = list
        .split(',')
        .map(str::trim)
        .filter(|key| !key.is_empty())
        .map(ToString::to_string)
        .collect::<Vec<_>>();
    (!keys.is_empty()).then_some(keys)
}

fn guide_section_chars(markdown: &str) -> std::collections::HashMap<String, usize> {
    let markdown = strip_frontmatter(markdown);
    let mut sections = std::collections::HashMap::new();
    let mut current: Option<(String, String)> = None;
    let mut fenced = false;

    for line in markdown.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
            fenced = !fenced;
            if let Some((_, body)) = &mut current {
                body.push('\n');
            }
            continue;
        }
        if !fenced && let Some(heading) = trimmed.strip_prefix("## ") {
            finalize_section(&mut sections, current.take());
            if let Some(captures) = SECTION_RE.captures(heading) {
                let number = captures
                    .get(1)
                    .map_or("", |value| value.as_str())
                    .to_string();
                let title = captures
                    .get(2)
                    .map_or("", |value| value.as_str())
                    .trim()
                    .to_string();
                current = Some((number, title));
            }
        } else if let Some((_, body)) = &mut current
            && !fenced
        {
            body.push_str(trimmed);
            body.push('\n');
        }
    }
    finalize_section(&mut sections, current);
    sections
}

fn finalize_section(
    sections: &mut std::collections::HashMap<String, usize>,
    current: Option<(String, String)>,
) {
    if let Some((number, body)) = current {
        sections.insert(number, count_body_chars(&body));
    }
}

fn count_body_chars(body: &str) -> usize {
    body.lines()
        .filter(|line| {
            let trimmed = line.trim();
            !trimmed.starts_with("|---") && !trimmed.starts_with("| ---")
        })
        .flat_map(|line| line.chars())
        .filter(|character| {
            !character.is_whitespace()
                && !matches!(
                    character,
                    '|' | '*' | '_' | '`' | '#' | '>' | '[' | ']' | '(' | ')'
                )
        })
        .count()
}

fn guide_lines(markdown: &str) -> Vec<String> {
    let markdown = strip_frontmatter(markdown);
    let mut lines = Vec::new();
    let mut fenced = false;
    for line in markdown.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
            fenced = !fenced;
            continue;
        }
        if fenced || trimmed.is_empty() || trimmed == "---" {
            continue;
        }
        if trimmed.starts_with('|') {
            // Table rows carry source references and bullet fragments, not headings.
            continue;
        }
        lines.push(trimmed.to_string());
    }
    lines
}

fn best_line(number: &str, title: &str, lines: &[String]) -> (f64, String) {
    let number_re = Regex::new(&regex::escape(number)).expect("valid section number regex");
    let target = normalize(&format!("{number} {title}"));
    let mut best_score = 0.0;
    let mut best_text = String::new();
    for line in lines {
        if !contains_number(line, &number_re) {
            continue;
        }
        let score = sequence_ratio(&target, &normalize(line));
        if score > best_score {
            best_score = score;
            best_text = line.clone();
        }
    }
    (best_score, best_text)
}

fn contains_number(line: &str, number_re: &Regex) -> bool {
    for found in number_re.find_iter(line) {
        let before = line[..found.start()].chars().next_back();
        let after = line[found.end()..].chars().next();
        let before_ok =
            !before.is_some_and(|character| character.is_ascii_digit() || character == '.');
        let after_ok =
            !after.is_some_and(|character| character.is_ascii_digit() || character == '.');
        if before_ok && after_ok {
            return true;
        }
    }
    false
}

fn strip_frontmatter(markdown: &str) -> &str {
    let Some(rest) = markdown.strip_prefix("---\n") else {
        return markdown;
    };
    let Some((_, body)) = rest.split_once("\n---\n") else {
        return markdown;
    };
    body
}

fn sequence_ratio(left: &str, right: &str) -> f64 {
    let left: Vec<char> = left.chars().collect();
    let right: Vec<char> = right.chars().collect();
    if left.is_empty() && right.is_empty() {
        return 1.0;
    }
    let mut previous = vec![0usize; right.len() + 1];
    let mut current = vec![0usize; right.len() + 1];
    for left_char in &left {
        for (index, right_char) in right.iter().enumerate() {
            current[index + 1] = if left_char == right_char {
                previous[index] + 1
            } else {
                previous[index + 1].max(current[index])
            };
        }
        std::mem::swap(&mut previous, &mut current);
    }
    let lcs = previous[right.len()];
    (2.0 * lcs as f64) / ((left.len() + right.len()) as f64)
}

fn normalize(text: &str) -> String {
    let mut normalized = String::with_capacity(text.len());
    for character in text.chars() {
        let character = match character {
            'ı' | 'İ' | 'I' => 'i',
            other => other.to_lowercase().next().unwrap_or(other),
        };
        if character.is_alphanumeric() || character.is_whitespace() {
            normalized.push(character);
        } else {
            normalized.push(' ');
        }
    }
    normalized.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::inventory::{Heading, PageRow, SourceInventory, Totals};

    fn inventory() -> Inventory {
        Inventory {
            generated_at: String::new(),
            low_text_threshold: 120,
            sources: vec![SourceInventory {
                path: String::new(),
                filename: "PD0.pdf".to_string(),
                title: String::new(),
                pages: 10,
                chars: 0,
                words: 0,
                low_text_threshold: 120,
                low_text_pages: vec![1],
                headings: vec![
                    Heading {
                        heading: "0.1 Giriş ve amaçlar".to_string(),
                        page: 3,
                    },
                    Heading {
                        heading: "0.2 Programlama dili nedir?".to_string(),
                        page: 10,
                    },
                ],
                page_map: vec![PageRow {
                    page: 1,
                    first_line: String::new(),
                    chars: 0,
                    low_text: true,
                }],
            }],
            totals: Totals {
                pages: 10,
                words: 0,
                low_text_pages: 1,
            },
        }
    }

    #[test]
    fn passes_when_all_sections_are_named() {
        let body = "Bu bölüm sınav için gerekli tanımı, mekanizmayı ve kritik ayrımı açıklar. Bu bölüm sınav için gerekli tanımı, mekanizmayı ve kritik ayrımı açıklar. Bu bölüm sınav için gerekli tanımı, mekanizmayı ve kritik ayrımı açıklar. Bu bölüm sınav için gerekli tanımı, mekanizmayı ve kritik ayrımı açıklar. Bu bölüm sınav için gerekli tanımı, mekanizmayı ve kritik ayrımı açıklar. Bu bölüm sınav için gerekli tanımı, mekanizmayı ve kritik ayrımı açıklar. Bu bölüm sınav için gerekli tanımı, mekanizmayı ve kritik ayrımı açıklar. Bu bölüm sınav için gerekli tanımı, mekanizmayı ve kritik ayrımı açıklar. ";
        let markdown = format!(
            "# Bölüm 0\n\n## 0.1 Giriş ve amaçlar\n\n{body}\n\n## 0.2 Programlama dili nedir?\n\n{body}\n"
        );
        let report = check_coverage(&inventory(), &markdown, DEFAULT_THRESHOLD);
        assert!(report.passed);
        assert_eq!(report.total, 2);
    }

    #[test]
    fn fails_when_a_section_is_missing() {
        let markdown = "# Bölüm 0\n\n## 0.1 Giriş ve amaçlar\n";
        let report = check_coverage(&inventory(), markdown, DEFAULT_THRESHOLD);
        assert!(!report.passed);
        assert_eq!(report.missing, 1);
    }

    #[test]
    fn tolerates_a_longer_guide_title() {
        let body = "Bu bölüm sınav için gerekli tanımı, mekanizmayı ve kritik ayrımı açıklar. Bu bölüm sınav için gerekli tanımı, mekanizmayı ve kritik ayrımı açıklar. Bu bölüm sınav için gerekli tanımı, mekanizmayı ve kritik ayrımı açıklar. Bu bölüm sınav için gerekli tanımı, mekanizmayı ve kritik ayrımı açıklar. Bu bölüm sınav için gerekli tanımı, mekanizmayı ve kritik ayrımı açıklar. Bu bölüm sınav için gerekli tanımı, mekanizmayı ve kritik ayrımı açıklar. Bu bölüm sınav için gerekli tanımı, mekanizmayı ve kritik ayrımı açıklar. Bu bölüm sınav için gerekli tanımı, mekanizmayı ve kritik ayrımı açıklar. ";
        let markdown = format!(
            "# Bölüm 1\n\n## 1.5 Dil kategorileri ve programlama paradigmaları\n\n{body}\n## 0.1 Giriş ve amaçlar\n\n{body}\n## 0.2 Programlama dili nedir?\n\n{body}\n"
        );
        let report = check_coverage(&inventory(), &markdown, DEFAULT_THRESHOLD);
        assert!(report.passed);
    }

    #[test]
    fn covers_metadata_claims_non_a_g_sections() {
        let body = "Bu bölüm sınav için gerekli tanımı, mekanizmayı ve kritik ayrımı açıklar. Bu bölüm sınav için gerekli tanımı, mekanizmayı ve kritik ayrımı açıklar. Bu bölüm sınav için gerekli tanımı, mekanizmayı ve kritik ayrımı açıklar. Bu bölüm sınav için gerekli tanımı, mekanizmayı ve kritik ayrımı açıklar. Bu bölüm sınav için gerekli tanımı, mekanizmayı ve kritik ayrımı açıklar. Bu bölüm sınav için gerekli tanımı, mekanizmayı ve kritik ayrımı açıklar. Bu bölüm sınav için gerekli tanımı, mekanizmayı ve kritik ayrımı açıklar. Bu bölüm sınav için gerekli tanımı, mekanizmayı ve kritik ayrımı açıklar. ";
        let markdown = format!("<!-- covers: PD0:0.1, PD0:0.2 -->\n## Foundations\n\n{body}\n");
        let report = check_coverage(&inventory(), &markdown, DEFAULT_THRESHOLD);
        assert!(report.passed, "{report:?}");
        assert_eq!(report.unknown_claims.len(), 0);
    }

    #[test]
    fn rejects_unknown_covers_claims() {
        let markdown = "<!-- covers: PD0:9.9 -->\n## Foundations\n\nshort\n";
        let report = check_coverage(&inventory(), markdown, DEFAULT_THRESHOLD);
        assert!(!report.passed);
        assert_eq!(report.unknown_claims, vec!["PD0:9.9"]);
    }

    #[test]
    fn ignores_headings_inside_code_fences() {
        let markdown = "```\n## 0.1 Giriş ve amaçlar\n## 0.2 Programlama dili nedir?\n```\n";
        let report = check_coverage(&inventory(), markdown, DEFAULT_THRESHOLD);
        assert!(!report.passed);
    }
}
