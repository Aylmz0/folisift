//! Output-intent recommendations.
//!
//! The intent is the reader task, not a typographic preset. It determines the
//! genre/structure and a page budget; the host still writes the content.

use serde::Serialize;

pub const OUTPUT_INTENTS: &[&str] = &[
    "exam-summary",
    "exam-revision",
    "concept-summary",
    "study",
    "deep",
    "reference",
    "practice",
    "one-page",
];

#[derive(Debug, Clone, Serialize)]
pub struct IntentRecommendation {
    pub intent: String,
    pub style: String,
    pub structure: String,
    pub target_pages: String,
    pub why: String,
    pub required_inputs: Vec<&'static str>,
}

pub fn recommend(intent: &str) -> Option<IntentRecommendation> {
    let (style, structure, target_pages, why) = match intent {
        "exam-summary" => (
            "revision",
            "revision-sheet",
            "2-4",
            "Sınav öncesi taranabilir, tablo/harita/hatırlama odaklı özet.",
        ),
        "exam-revision" => (
            "revision",
            "revision-sheet",
            "6-10",
            "Tüm konuları kapsayan, kart ve tablo ağırlıklı sınav tekrarı.",
        ),
        "concept-summary" => (
            "reader",
            "concept-summary",
            "4-8",
            "Açıklayıcı prose, mekanizma, örnek ve karşılaştırma tablosu ile anlama özeti.",
        ),
        "study" => (
            "reader",
            "a-g",
            "10-18",
            "Konuyu öğrenmek için bölüm akışı, prose ve destekleyici görseller.",
        ),
        "deep" => (
            "reader",
            "a-g",
            "16-28",
            "Derin çalışma; mekanizma, varsayım, sınır ve bağlantılar.",
        ),
        "reference" => (
            "reference",
            "formula-sheet",
            "2-8",
            "Hızlı başvuru; formül, sembol, birim ve koşullar.",
        ),
        "practice" => (
            "workbook",
            "workbook",
            "6-12",
            "Problem çözme; çalışma alanı, kademeli örnek ve cevap anahtarı.",
        ),
        "one-page" => (
            "revision",
            "knowledge-organiser",
            "1",
            "Tek sayfalık bilgi organizatörü; 10–15 terim ve tek harita.",
        ),
        _ => return None,
    };
    Some(IntentRecommendation {
        intent: intent.to_string(),
        style: style.to_string(),
        structure: structure.to_string(),
        target_pages: target_pages.to_string(),
        why: why.to_string(),
        required_inputs: required_inputs(intent),
    })
}

fn required_inputs(intent: &str) -> Vec<&'static str> {
    match intent {
        "exam-summary" | "exam-revision" => vec!["source page scope", "exam date or urgency"],
        "concept-summary" | "study" | "deep" => vec!["source page scope", "prior knowledge level"],
        "reference" => vec!["topic scope", "allowed exam aids"],
        "practice" => vec!["problem source", "answer availability"],
        "one-page" => vec!["one unit/topic"],
        _ => Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn intents_map_to_the_right_structure_and_budget() {
        let exam = recommend("exam-summary").expect("exam");
        assert_eq!(exam.style, "revision");
        assert_eq!(exam.structure, "revision-sheet");
        assert_eq!(exam.target_pages, "2-4");
        let concept = recommend("concept-summary").expect("concept");
        assert_eq!(concept.style, "reader");
        assert_eq!(concept.structure, "concept-summary");
        assert_eq!(concept.target_pages, "4-8");
        assert!(recommend("unknown").is_none());
    }
}
