//! Extract recall/answer pairs into an Anki-compatible CSV.
//!
//! Review intervals are guide-level metadata. The CSV carries them as Anki
//! tags (`review-1d`, `review-3d`, ...); the review application owns the
//! mutable state (`last_reviewed`, confidence) and schedules the next card.

use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct Card {
    pub front: String,
    pub back: String,
    pub source: String,
    pub tags: Vec<String>,
}

/// Validate and normalize review intervals: 0-10 unique positive day counts.
///
/// An empty list is an explicit choice: cards are still exported, without
/// `review-*` tags.
pub fn normalize_review_intervals(mut intervals: Vec<u32>) -> Result<Vec<u32>, String> {
    if intervals.len() > 10 {
        return Err("review intervals must contain at most 10 entries".to_string());
    }
    if intervals.iter().any(|days| *days == 0 || *days > 365) {
        return Err("review intervals must be between 1 and 365 days".to_string());
    }
    intervals.sort_unstable();
    intervals.dedup();
    Ok(intervals)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CardPolicy {
    pub include_definitions: bool,
}

impl Default for CardPolicy {
    fn default() -> Self {
        Self {
            include_definitions: true,
        }
    }
}

pub fn cards_from_markdown(markdown: &str, slug: &str, review_intervals: &[u32]) -> Vec<Card> {
    cards_from_markdown_opts(markdown, slug, review_intervals, false)
}

pub fn cards_from_markdown_policy(
    markdown: &str,
    slug: &str,
    review_intervals: &[u32],
    policy: CardPolicy,
) -> Vec<Card> {
    cards_from_markdown_opts(markdown, slug, review_intervals, policy.include_definitions)
}

pub fn cards_from_markdown_opts(
    markdown: &str,
    slug: &str,
    review_intervals: &[u32],
    include_definitions: bool,
) -> Vec<Card> {
    let body = strip_frontmatter(markdown);
    let blocks = blocks(body);
    let recalls = blocks
        .iter()
        .filter(|(kind, _, _)| kind == "recall")
        .collect::<Vec<_>>();
    let first_recall_line = recalls.first().map(|(_, _, line)| *line).unwrap_or(0);
    let answers = blocks
        .iter()
        .filter(|(kind, _, line)| kind == "answer" && *line > first_recall_line)
        .collect::<Vec<_>>();
    let mut cards = Vec::new();
    for (front, back) in recalls.iter().zip(answers) {
        let front = front.1.trim();
        let back = back.1.trim();
        if front.is_empty() || back.is_empty() {
            continue;
        }
        let source = source_of(front)
            .or_else(|| source_of(back))
            .unwrap_or_default();
        let mut tags = vec![slug.to_string(), "recall".to_string()];
        for interval in review_intervals {
            tags.push(format!("review-{interval}d"));
        }
        cards.push(Card {
            front: front.to_string(),
            back: back.to_string(),
            source,
            tags,
        });
    }
    if include_definitions {
        for (kind, body, _) in blocks
            .iter()
            .filter(|(kind, _, _)| kind == "definition" || kind == "must-know")
        {
            let body = body.trim();
            if body.is_empty() {
                continue;
            }
            let front = body
                .lines()
                .find(|line| !line.trim().is_empty())
                .unwrap_or(body)
                .trim()
                .chars()
                .take(160)
                .collect::<String>();
            let source = source_of(body).unwrap_or_default();
            let mut tags = vec![slug.to_string(), kind.clone()];
            for interval in review_intervals {
                tags.push(format!("review-{interval}d"));
            }
            cards.push(Card {
                front,
                back: body.to_string(),
                source,
                tags,
            });
        }
    }
    cards
}

pub fn to_csv(cards: &[Card]) -> String {
    let mut out = String::from("front,back,tags,source\n");
    for card in cards {
        out.push_str(&format!(
            "{},{},{},{}\n",
            csv(&card.front),
            csv(&card.back),
            csv(&card.tags.join(" ")),
            csv(&card.source)
        ));
    }
    out
}

fn blocks(markdown: &str) -> Vec<(String, String, usize)> {
    let mut out = Vec::new();
    let lines = markdown.lines().collect::<Vec<_>>();
    let mut index = 0;
    while index < lines.len() {
        let line = lines[index].trim();
        let Some(rest) = line.strip_prefix(":::") else {
            index += 1;
            continue;
        };
        let kind = rest.split_whitespace().next().unwrap_or_default();
        if !matches!(kind, "recall" | "answer" | "definition" | "must-know") {
            index += 1;
            continue;
        }
        let start = index + 1;
        index += 1;
        let mut body = Vec::new();
        while index < lines.len() && lines[index].trim() != ":::" {
            body.push(lines[index]);
            index += 1;
        }
        out.push((kind.to_string(), body.join("\n"), start));
        index += 1;
    }
    out
}

fn source_of(text: &str) -> Option<String> {
    let start = text.rfind('[')?;
    let end = text[start..].find(']')?;
    Some(text[start + 1..start + end].to_string())
}

fn csv(value: &str) -> String {
    if value.contains([',', '"', '\n']) {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_string()
    }
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
    fn pairs_recall_and_answer_blocks() {
        let markdown = "---\ntitle: X\n---\n\n::: recall\nWhat is X? [SRC1, s. 2]\n:::\n\n::: answer\nX is Y. [SRC1, s. 2]\n:::\n";
        let cards = cards_from_markdown(markdown, "demo", &[1, 3, 7]);
        assert_eq!(cards.len(), 1);
        assert_eq!(cards[0].front, "What is X? [SRC1, s. 2]");
        assert_eq!(cards[0].source, "SRC1, s. 2");
        assert!(cards[0].tags.contains(&"review-3d".to_string()));
        assert!(to_csv(&cards).starts_with("front,back,tags,source"));
    }

    #[test]
    fn pairs_multiple_recalls_with_answers_in_order() {
        let markdown = "::: recall\nQ1 [SRC1, s. 1]\n:::\n\n::: recall\nQ2 [SRC1, s. 2]\n:::\n\n::: answer\nA1 [SRC1, s. 1]\n:::\n\n::: answer\nA2 [SRC1, s. 2]\n:::\n";
        let cards = cards_from_markdown(markdown, "demo", &[1]);
        assert_eq!(cards.len(), 2);
        assert_eq!(cards[0].back, "A1 [SRC1, s. 1]");
        assert_eq!(cards[1].back, "A2 [SRC1, s. 2]");
    }

    #[test]
    fn ignores_answers_before_the_recall_section() {
        let markdown = "::: answer label=\"EXAMPLE\"\nExample answer. [SRC1, s. 1]\n:::\n\n::: recall\nQ1 [SRC1, s. 2]\n:::\n\n::: answer label=\"A1\"\nA1 [SRC1, s. 2]\n:::\n";
        let cards = cards_from_markdown(markdown, "demo", &[1]);
        assert_eq!(cards.len(), 1);
        assert_eq!(cards[0].back, "A1 [SRC1, s. 2]");
    }

    #[test]
    fn definitions_are_opt_in_cards() {
        let markdown = "::: definition label=\"X\"\n**X:** defined term. [SRC1, s. 1]\n:::\n";
        assert!(cards_from_markdown(markdown, "demo", &[1]).is_empty());
        let cards = cards_from_markdown_opts(markdown, "demo", &[1], true);
        assert_eq!(cards.len(), 1);
        assert!(cards[0].front.contains("X:"));
        assert!(cards[0].tags.contains(&"definition".to_string()));
    }

    #[test]
    fn the_default_card_policy_includes_definitions() {
        let markdown = "::: recall\nQ1 [SRC1, s. 1]\n:::\n\n::: answer\nA1 [SRC1, s. 1]\n:::\n\n::: definition label=\"X\"\n**X:** defined term. [SRC1, s. 1]\n:::\n";
        let policy = CardPolicy::default();
        assert!(policy.include_definitions);
        let cards = cards_from_markdown_policy(markdown, "demo", &[1], policy);
        assert_eq!(cards.len(), 2);
        assert_eq!(
            cards_from_markdown_opts(markdown, "demo", &[1], true).len(),
            cards.len()
        );
        let excluded = cards_from_markdown_policy(
            markdown,
            "demo",
            &[1],
            CardPolicy {
                include_definitions: false,
            },
        );
        assert_eq!(excluded.len(), 1);
    }

    #[test]
    fn review_intervals_are_validated_and_normalized() {
        assert_eq!(
            normalize_review_intervals(vec![7, 1, 3, 3]).unwrap(),
            vec![1, 3, 7]
        );
        assert_eq!(
            normalize_review_intervals(vec![]).unwrap(),
            Vec::<u32>::new()
        );
        assert!(normalize_review_intervals(vec![0]).is_err());
        assert!(normalize_review_intervals(vec![366]).is_err());
    }
}
