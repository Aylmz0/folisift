use std::fmt::Write as _;
use std::path::Path;
use std::sync::LazyLock;

use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::error::{Result, io};
use crate::inventory::{Inventory, SourceInventory};

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
    pub canonical_total: usize,
    pub claimed: usize,
    pub gate_total: usize,
    pub missing: usize,
    pub missing_from_gate: Vec<String>,
    pub empty_sections: Vec<String>,
    pub weak_sections: Vec<String>,
    pub unknown_claims: Vec<String>,
    pub duplicate_claims: Vec<String>,
    pub audit_missing: bool,
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

/// Candidate sections: the source's own topic list when available, otherwise
/// the extracted headings. Canonical sections survive styled/wrapped headings.
fn source_candidates(source: &SourceInventory) -> Vec<(String, String, usize)> {
    if !source.sections.is_empty() {
        return source
            .sections
            .iter()
            .map(|section| {
                let page = source
                    .headings
                    .iter()
                    .find(|heading| {
                        SECTION_RE
                            .captures(heading.heading.trim())
                            .is_some_and(|captures| captures[1] == section.number)
                    })
                    .map_or(0, |heading| heading.page);
                (section.number.clone(), section.title.clone(), page)
            })
            .collect();
    }
    source
        .headings
        .iter()
        .filter_map(|heading| {
            let captures = SECTION_RE.captures(heading.heading.trim())?;
            Some((
                captures[1].to_string(),
                captures[2].trim().to_string(),
                heading.page,
            ))
        })
        .collect()
}

/// Compare every numbered source subsection with the guide's Markdown lines.
pub fn check_coverage(
    inventory: &Inventory,
    guide_markdown: &str,
    threshold: f64,
) -> CoverageReport {
    let lines = guide_lines(guide_markdown);
    let sections_by_number = guide_section_chars(guide_markdown);
    let claims = scan_claims(guide_markdown);
    let claim_chars = &claims.chars;
    let audit_missing = !has_audit_section(guide_markdown);
    let mut known_claims = std::collections::HashSet::new();
    for source in &inventory.sources {
        let id = source.filename.trim_end_matches(".pdf");
        for (number, _, _) in source_candidates(source) {
            known_claims.insert(format!("{id}:{number}"));
        }
    }
    let unknown_claims = claim_chars
        .keys()
        .filter(|key| !known_claims.contains(*key))
        .cloned()
        .collect::<Vec<_>>();
    let mut sources = Vec::with_capacity(inventory.sources.len());
    let mut total = 0;
    let mut claimed_total = 0;
    let mut gate_total = 0;
    let mut missing = 0;
    let mut missing_from_gate = Vec::new();
    let mut empty_sections = Vec::new();
    let mut weak_sections = Vec::new();

    for source in &inventory.sources {
        let mut sections = Vec::new();
        let mut seen = std::collections::HashSet::new();
        for (number, title, page) in source_candidates(source) {
            if !seen.insert(number.clone()) {
                continue;
            }
            total += 1;
            let source_id = source.filename.trim_end_matches(".pdf");
            let claim_key = format!("{source_id}:{number}");
            let claimed = claim_chars.get(&claim_key).copied();
            if claimed.is_some() || sections_by_number.contains_key(&number) {
                claimed_total += 1;
            }
            let (score, matched) = if claimed.is_some() {
                (1.0, format!("<!-- covers: {claim_key} -->"))
            } else {
                best_line(&number, &title, &lines)
            };
            let heading_ok = claimed.is_some() || score >= threshold;
            if heading_ok {
                gate_total += 1;
            } else {
                missing += 1;
                missing_from_gate.push(number.clone());
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
                page,
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
        passed: missing == 0
            && empty_sections.is_empty()
            && unknown_claims.is_empty()
            && claims.duplicates.is_empty()
            && !audit_missing,
        threshold,
        total,
        canonical_total: total,
        claimed: claimed_total,
        gate_total,
        missing,
        missing_from_gate,
        empty_sections,
        weak_sections,
        unknown_claims,
        duplicate_claims: claims.duplicates,
        audit_missing,
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
                let page = if section.page == 0 {
                    "—".to_string()
                } else {
                    section.page.to_string()
                };
                let _ = writeln!(
                    out,
                    "| {} {} | {} | {} | {} | {} |",
                    section.number, section.title, page, status, section.body_chars, matched
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
        let _ = writeln!(out, "- Kanonik toplam: **{}**", self.canonical_total);
        let _ = writeln!(
            out,
            "- Kılavuzda başlığı/claim'i olan: **{}**",
            self.claimed
        );
        let _ = writeln!(out, "- Gate içinde eşleşen: **{}**", self.gate_total);
        let _ = writeln!(out, "- Eksik/weak match: **{}**", self.missing);
        if !self.missing_from_gate.is_empty() {
            let _ = writeln!(
                out,
                "- Gate dışında kalan: **{}**",
                self.missing_from_gate.join(", ")
            );
        }
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

#[derive(Default)]
struct ClaimScan {
    chars: std::collections::HashMap<String, usize>,
    duplicates: Vec<String>,
}

fn scan_claims(markdown: &str) -> ClaimScan {
    let markdown = strip_frontmatter(markdown);
    let mut scan = ClaimScan::default();
    let mut pending: Vec<String> = Vec::new();
    let mut current: Option<(Vec<String>, String)> = None;
    let mut fenced = false;

    fn finalize(scan: &mut ClaimScan, current: Option<(Vec<String>, String)>) {
        if let Some((keys, body)) = current {
            let chars = count_body_chars(&body);
            for key in keys {
                if scan.chars.insert(key.clone(), chars).is_some() {
                    scan.duplicates.push(key);
                }
            }
        }
    }

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
        if trimmed.starts_with("## ") {
            finalize(&mut scan, current.take());
            if !pending.is_empty() {
                current = Some((std::mem::take(&mut pending), String::new()));
            }
            continue;
        }
        if trimmed.starts_with('#') {
            // A subheading stays inside the current section body.
            continue;
        }
        if let Some((_, body)) = &mut current {
            body.push_str(trimmed);
            body.push('\n');
        }
    }
    finalize(&mut scan, current);
    scan.duplicates.sort();
    scan.duplicates.dedup();
    scan
}

fn has_audit_section(markdown: &str) -> bool {
    let markdown = strip_frontmatter(markdown);
    let mut fenced = false;
    for line in markdown.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
            fenced = !fenced;
            continue;
        }
        if fenced {
            continue;
        }
        if trimmed.eq_ignore_ascii_case("<!-- audit -->") {
            return true;
        }
        if trimmed.starts_with('#') {
            let normalized = normalize(trimmed.trim_start_matches('#').trim());
            if normalized.contains("kapsam denetimi")
                || normalized.contains("coverage audit")
                || normalized == "audit"
                || normalized.ends_with(" audit")
            {
                return true;
            }
        }
    }
    false
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
                sections: vec![],
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
            "<!-- audit -->\n# Bölüm 0\n\n## 0.1 Giriş ve amaçlar\n\n{body}\n\n## 0.2 Programlama dili nedir?\n\n{body}\n"
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
            "<!-- audit -->\n# Bölüm 1\n\n## 1.5 Dil kategorileri ve programlama paradigmaları\n\n{body}\n## 0.1 Giriş ve amaçlar\n\n{body}\n## 0.2 Programlama dili nedir?\n\n{body}\n"
        );
        let report = check_coverage(&inventory(), &markdown, DEFAULT_THRESHOLD);
        assert!(report.passed);
    }

    #[test]
    fn covers_metadata_claims_non_a_g_sections() {
        let body = "Bu bölüm sınav için gerekli tanımı, mekanizmayı ve kritik ayrımı açıklar. Bu bölüm sınav için gerekli tanımı, mekanizmayı ve kritik ayrımı açıklar. Bu bölüm sınav için gerekli tanımı, mekanizmayı ve kritik ayrımı açıklar. Bu bölüm sınav için gerekli tanımı, mekanizmayı ve kritik ayrımı açıklar. Bu bölüm sınav için gerekli tanımı, mekanizmayı ve kritik ayrımı açıklar. Bu bölüm sınav için gerekli tanımı, mekanizmayı ve kritik ayrımı açıklar. Bu bölüm sınav için gerekli tanımı, mekanizmayı ve kritik ayrımı açıklar. Bu bölüm sınav için gerekli tanımı, mekanizmayı ve kritik ayrımı açıklar. ";
        let markdown = format!(
            "<!-- audit -->\n<!-- covers: PD0:0.1, PD0:0.2 -->\n## Foundations\n\n{body}\n"
        );
        let report = check_coverage(&inventory(), &markdown, DEFAULT_THRESHOLD);
        assert!(report.passed, "{report:?}");
        assert_eq!(report.unknown_claims.len(), 0);
    }

    #[test]
    fn covers_body_includes_subheadings() {
        let body = "Bu bölüm sınav için gerekli tanımı, mekanizmayı ve kritik ayrımı açıklar. Bu bölüm sınav için gerekli tanımı, mekanizmayı ve kritik ayrımı açıklar. Bu bölüm sınav için gerekli tanımı, mekanizmayı ve kritik ayrımı açıklar. Bu bölüm sınav için gerekli tanımı, mekanizmayı ve kritik ayrımı açıklar. Bu bölüm sınav için gerekli tanımı, mekanizmayı ve kritik ayrımı açıklar. Bu bölüm sınav için gerekli tanımı, mekanizmayı ve kritik ayrımı açıklar. Bu bölüm sınav için gerekli tanımı, mekanizmayı ve kritik ayrımı açıklar. Bu bölüm sınav için gerekli tanımı, mekanizmayı ve kritik ayrımı açıklar. ";
        let markdown = format!(
            "<!-- audit -->\n<!-- covers: PD0:0.1, PD0:0.2 -->\n## Foundations\n\n### Subheading\n\n{body}\n"
        );
        let report = check_coverage(&inventory(), &markdown, DEFAULT_THRESHOLD);
        assert!(report.passed, "{report:?}");
        assert!(report.sources[0].sections[0].body_chars > MIN_SECTION_CHARS);
    }

    #[test]
    fn rejects_unknown_covers_claims() {
        let markdown = "<!-- covers: PD0:9.9 -->\n## Foundations\n\nshort\n";
        let report = check_coverage(&inventory(), markdown, DEFAULT_THRESHOLD);
        assert!(!report.passed);
        assert_eq!(report.unknown_claims, vec!["PD0:9.9"]);
    }

    #[test]
    fn checked_in_fixtures_seed_duplicate_and_missing_audit_defects() {
        let duplicate = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/fixtures/covers-duplicate.md"
        ));
        let report = check_coverage(&inventory(), duplicate, DEFAULT_THRESHOLD);
        assert_eq!(report.duplicate_claims, vec!["PD0:0.1"]);
        assert!(!report.passed);

        let missing = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/fixtures/audit-missing.md"
        ));
        let report = check_coverage(&inventory(), missing, DEFAULT_THRESHOLD);
        assert!(report.audit_missing);
        assert!(!report.passed);
    }

    #[test]
    fn rejects_duplicate_covers_claims() {
        let body = "Bu bölüm sınav için gerekli tanımı, mekanizmayı ve kritik ayrımı açıklar. Bu bölüm sınav için gerekli tanımı, mekanizmayı ve kritik ayrımı açıklar. Bu bölüm sınav için gerekli tanımı, mekanizmayı ve kritik ayrımı açıklar. Bu bölüm sınav için gerekli tanımı, mekanizmayı ve kritik ayrımı açıklar. Bu bölüm sınav için gerekli tanımı, mekanizmayı ve kritik ayrımı açıklar. Bu bölüm sınav için gerekli tanımı, mekanizmayı ve kritik ayrımı açıklar. Bu bölüm sınav için gerekli tanımı, mekanizmayı ve kritik ayrımı açıklar. Bu bölüm sınav için gerekli tanımı, mekanizmayı ve kritik ayrımı açıklar. ";
        let markdown = format!(
            "<!-- audit -->\n<!-- covers: PD0:0.1 -->\n## A\n\n{body}\n<!-- covers: PD0:0.1 -->\n## B\n\n{body}\n"
        );
        let report = check_coverage(&inventory(), &markdown, DEFAULT_THRESHOLD);
        assert!(!report.passed);
        assert_eq!(report.duplicate_claims, vec!["PD0:0.1"]);
    }

    #[test]
    fn requires_an_audit_section() {
        let markdown = "<!-- covers: PD0:0.1, PD0:0.2 -->\n## Foundations\n\nShort.\n";
        let report = check_coverage(&inventory(), markdown, DEFAULT_THRESHOLD);
        assert!(!report.passed);
        assert!(report.audit_missing);
    }

    #[test]
    fn ignores_headings_inside_code_fences() {
        let markdown = "```\n## 0.1 Giriş ve amaçlar\n## 0.2 Programlama dili nedir?\n```\n";
        let report = check_coverage(&inventory(), markdown, DEFAULT_THRESHOLD);
        assert!(!report.passed);
    }

    #[test]
    fn canonical_toc_sections_are_the_coverage_set() {
        let body = "Bu bölüm sınav için gerekli tanımı, mekanizmayı ve kritik ayrımı açıklar. Bu bölüm sınav için gerekli tanımı, mekanizmayı ve kritik ayrımı açıklar. Bu bölüm sınav için gerekli tanımı, mekanizmayı ve kritik ayrımı açıklar. Bu bölüm sınav için gerekli tanımı, mekanizmayı ve kritik ayrımı açıklar. Bu bölüm sınav için gerekli tanımı, mekanizmayı ve kritik ayrımı açıklar. Bu bölüm sınav için gerekli tanımı, mekanizmayı ve kritik ayrımı açıklar. Bu bölüm sınav için gerekli tanımı, mekanizmayı ve kritik ayrımı açıklar. Bu bölüm sınav için gerekli tanımı, mekanizmayı ve kritik ayrımı açıklar. ";
        let mut inv = inventory();
        inv.sources[0].sections = vec![
            crate::inventory::CanonicalSection {
                number: "0.1".to_string(),
                title: "Giriş ve amaçlar".to_string(),
                page: 2,
                origin: "toc".to_string(),
            },
            crate::inventory::CanonicalSection {
                number: "0.2".to_string(),
                title: "Programlama dili nedir?".to_string(),
                page: 3,
                origin: "toc".to_string(),
            },
            crate::inventory::CanonicalSection {
                number: "0.3".to_string(),
                title: "Biçimsel diller".to_string(),
                page: 4,
                origin: "toc".to_string(),
            },
        ];
        let markdown = format!(
            "<!-- audit -->\n## 0.1 Giriş ve amaçlar\n\n{body}\n## 0.2 Programlama dili nedir?\n\n{body}\n## 0.3 Biçimsel diller\n\n{body}\n"
        );
        let report = check_coverage(&inv, &markdown, DEFAULT_THRESHOLD);
        assert!(report.passed, "{report:?}");
        assert_eq!(report.total, 3);
        assert_eq!(report.canonical_total, 3);
        assert_eq!(report.claimed, 3);
        assert_eq!(report.gate_total, 3);
        assert_eq!(report.missing, 0);
    }

    #[test]
    fn canonical_toc_section_missing_from_guide_fails() {
        let mut inv = inventory();
        inv.sources[0].sections = vec![crate::inventory::CanonicalSection {
            number: "0.3".to_string(),
            title: "Biçimsel diller".to_string(),
            page: 4,
            origin: "toc".to_string(),
        }];
        let report = check_coverage(
            &inv,
            "<!-- audit -->\n## 0.1 Giriş ve amaçlar\n\nshort\n",
            DEFAULT_THRESHOLD,
        );
        assert!(!report.passed);
        assert_eq!(report.total, 1);
        assert_eq!(report.missing, 1);
    }
}
