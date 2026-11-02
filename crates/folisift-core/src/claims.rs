//! Deterministic, review-only claim extraction.
//!
//! A citation is traceability, not verification. This module finds high-signal
//! claims (numbers, dates, comparisons, causal statements) and whether they
//! carry a page reference. It never decides that a claim is true.

use std::sync::LazyLock;

use regex::Regex;
use serde::Serialize;

static PAGE_REF_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\[([^\[\],]+?)\s*,\s*(?:s|p)\.\s*(\d+)(?:\s*[–-]\s*(\d+))?\]")
        .expect("valid page reference regex")
});
static HIGH_SIGNAL_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?i)(\d|%|artar|azalır|daha (az|çok|fazla)|önce|sonra|çünkü|bu yüzden|nedeniyle|before|after|more|less|increase|decrease|because|therefore|percent|between|since)",
    )
    .expect("valid claim regex")
});

#[derive(Debug, Clone, Serialize)]
pub struct ClaimRef {
    pub source: String,
    pub page: usize,
    pub end_page: Option<usize>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Claim {
    pub line: usize,
    pub kind: String,
    pub text: String,
    pub references: Vec<ClaimRef>,
    pub referenced: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct ClaimReport {
    pub total: usize,
    pub referenced: usize,
    pub unreferenced: usize,
    pub claims: Vec<Claim>,
}

impl ClaimReport {
    pub fn write_markdown(&self, path: &std::path::Path) -> crate::Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|error| crate::error::io(parent, error))?;
        }
        let mut out = String::from("# Claim spot-check\n\n");
        out.push_str("A citation is traceability, not verification. Check the subject, number/unit, date, causal wording and qualifiers against the cited page.\n\n");
        out.push_str(&format!(
            "High-signal claims: **{}** · referenced: **{}** · unreferenced: **{}**\n\n",
            self.total, self.referenced, self.unreferenced
        ));
        out.push_str("| Line | Kind | Source page(s) | Claim |\n|---:|---|---|---|\n");
        for claim in &self.claims {
            let text = claim.text.replace('|', "\\|");
            let text = text.chars().take(240).collect::<String>();
            let refs = if claim.references.is_empty() {
                "**missing**".to_string()
            } else {
                claim
                    .references
                    .iter()
                    .map(|reference| match reference.end_page {
                        Some(end) => format!("{} s. {}-{}", reference.source, reference.page, end),
                        None => format!("{} s. {}", reference.source, reference.page),
                    })
                    .collect::<Vec<_>>()
                    .join("; ")
            };
            out.push_str(&format!(
                "| {} | {} | {} | {} |\n",
                claim.line, claim.kind, refs, text
            ));
        }
        out.push_str("\nThis report is a reading aid, not a truth verdict.\n");
        std::fs::write(path, out).map_err(|error| crate::error::io(path, error))
    }
}

pub fn extract_claims(markdown: &str) -> ClaimReport {
    let body = strip_frontmatter(markdown);
    let mut claims = Vec::new();
    let mut fenced = false;
    let mut in_recall = false;
    let mut paragraph = String::new();
    let mut paragraph_line = 0;
    for (index, raw) in body.lines().enumerate() {
        let line = raw.trim();
        if line.starts_with("```") || line.starts_with("~~~") {
            flush_paragraph(&mut paragraph, paragraph_line, &mut claims);
            fenced = !fenced;
            continue;
        }
        if let Some(directive) = line.strip_prefix(":::") {
            flush_paragraph(&mut paragraph, paragraph_line, &mut claims);
            let directive = directive.trim();
            if directive.is_empty() {
                in_recall = false;
            } else if directive
                .split_whitespace()
                .next()
                .is_some_and(|kind| kind == "recall")
            {
                in_recall = true;
            }
            continue;
        }
        if fenced
            || in_recall
            || line.is_empty()
            || line.starts_with('#')
            || line.starts_with('|')
            || line.starts_with("<!--")
        {
            flush_paragraph(&mut paragraph, paragraph_line, &mut claims);
            continue;
        }
        if line.starts_with("- ")
            || line.starts_with("* ")
            || line.split_once(". ").is_some_and(|(prefix, _)| {
                prefix.chars().all(|character| character.is_ascii_digit())
            })
        {
            flush_paragraph(&mut paragraph, paragraph_line, &mut claims);
            push_claims(line, index + 1, &mut claims);
            continue;
        }
        if paragraph.is_empty() {
            paragraph_line = index + 1;
        } else {
            paragraph.push(' ');
        }
        paragraph.push_str(line);
    }
    flush_paragraph(&mut paragraph, paragraph_line, &mut claims);
    let referenced = claims.iter().filter(|claim| claim.referenced).count();
    ClaimReport {
        total: claims.len(),
        referenced,
        unreferenced: claims.len() - referenced,
        claims,
    }
}

fn flush_paragraph(paragraph: &mut String, line: usize, claims: &mut Vec<Claim>) {
    if paragraph.is_empty() {
        return;
    }
    push_claims(paragraph, line, claims);
    paragraph.clear();
}

fn push_claims(text: &str, line: usize, claims: &mut Vec<Claim>) {
    for sentence in split_sentences(text) {
        let text = sentence.trim();
        if text.chars().count() < 20 || !HIGH_SIGNAL_RE.is_match(text) {
            continue;
        }
        let references = PAGE_REF_RE
            .captures_iter(text)
            .filter_map(|capture| {
                Some(ClaimRef {
                    source: capture.get(1)?.as_str().to_string(),
                    page: capture.get(2)?.as_str().parse().ok()?,
                    end_page: capture
                        .get(3)
                        .and_then(|value| value.as_str().parse::<usize>().ok()),
                })
            })
            .collect::<Vec<_>>();
        let kind = classify(text);
        claims.push(Claim {
            line,
            kind,
            text: text.to_string(),
            referenced: !references.is_empty(),
            references,
        });
    }
}

fn classify(text: &str) -> String {
    let lower = text.to_lowercase();
    if lower.contains('%') || lower.chars().any(|character| character.is_ascii_digit()) {
        "number/date".to_string()
    } else if lower.contains("daha")
        || lower.contains("more")
        || lower.contains("less")
        || lower.contains("artar")
        || lower.contains("azalır")
    {
        "comparison".to_string()
    } else {
        "causal".to_string()
    }
}

fn split_sentences(line: &str) -> Vec<&str> {
    let mut parts = Vec::new();
    let mut start = 0;
    for (index, character) in line.char_indices() {
        if !matches!(character, '.' | '!' | '?') {
            continue;
        }
        let after = &line[index + character.len_utf8()..];
        let trimmed = after.trim_start();
        let next = trimmed.chars().next();
        if next.is_some_and(|value| value.is_uppercase() || matches!(value, '"' | '“')) {
            let end = index + character.len_utf8();
            parts.push(line[start..end].trim());
            start = end + (after.len() - trimmed.len());
        }
    }
    let tail = line[start..].trim();
    if !tail.is_empty() {
        parts.push(tail);
    }
    parts
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_high_signal_claims_and_missing_refs() {
        let markdown = "---\ntitle: X\n---\n\nRecall improves by 20% after one week [SRC1, s. 4].\nThe sample was small.\nThis increases retention [SRC1, s. 5].\n";
        let report = extract_claims(markdown);
        assert_eq!(report.total, 2);
        assert_eq!(report.referenced, 2);
        let markdown = "Recall improves by 20% after one week.\n";
        assert_eq!(extract_claims(markdown).unreferenced, 1);
    }

    #[test]
    fn accepts_unicode_source_ids() {
        let report = extract_claims("Retention increased by 20% [Ders_Notu_1_VİZE, s. 4].\n");
        assert_eq!(report.referenced, 1);
        assert_eq!(report.claims[0].references[0].source, "Ders_Notu_1_VİZE");
    }

    #[test]
    fn recall_questions_are_not_claims() {
        let report = extract_claims(
            "::: recall\nWhat changed by 20% after one week? [SRC1, s. 4]\n:::\n\n::: answer\nRetention increased by 20% after one week [SRC1, s. 4].\n:::\n",
        );
        assert_eq!(report.total, 1);
        assert!(report.claims[0].text.contains("Retention"));
    }

    #[test]
    fn markdown_report_shows_the_cited_page_or_a_missing_marker() {
        let report = extract_claims(
            "X increased by 20% after the intervention [SRC1, s. 4-5].\nRetention fell by 3% in the comparison group.\n",
        );
        let path = std::env::temp_dir().join(format!("folisift-claims-{}.md", std::process::id()));
        report.write_markdown(&path).expect("report");
        let text = std::fs::read_to_string(&path).expect("read report");
        let _ = std::fs::remove_file(path);
        assert!(text.contains("SRC1 s. 4-5"), "{text}");
        assert!(text.contains("**missing**"), "{text}");
    }
}
