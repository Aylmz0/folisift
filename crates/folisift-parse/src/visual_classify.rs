//! Content, relevance and reconstruction classification for visual candidates.
//!
//! The classifier is deliberately conservative. D2 is a diagram engine, not a
//! plotting library; a qualitative schematic is allowed only when the text
//! asks for a relation rather than exact values.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ClassifyResult {
    pub content_class: String,
    pub relevance: String,
    pub relevance_confidence: String,
    pub reconstruction: String,
    pub reconstruction_confidence: String,
    pub reasons: Vec<String>,
}

#[derive(Debug, Clone, Copy)]
pub struct ClassifyInput<'a> {
    /// Heading + caption seed. Title keywords drive diagram/chart decisions.
    pub title: &'a str,
    /// Bounded context + text inside the candidate.
    pub body: &'a str,
    /// Full page text, used only as a fallback for tiny figures whose own
    /// labels are not extracted.
    pub page_text: &'a str,
    pub table: bool,
    pub native_image: bool,
    pub vector_primitives: usize,
    pub low_text: bool,
    pub text_inside_chars: usize,
    pub decorative: bool,
}

const CHART_WORDS: &[&str] = &[
    "grafik",
    "plot",
    "scatter",
    "saçılım",
    "serpilme",
    "dağılım",
    "histogram",
    "roc",
    "eğri",
    "quantile",
    "kutu grafiği",
    "box plot",
    "regresyon",
    "bar chart",
    "pasta grafiği",
    "çizgi grafiği",
    "cluster",
    "küme",
    "kmeans",
    "k-means",
    "dbscan",
    "hiyerarşik",
    "hierarchical",
    "demetleme",
    "clustering",
];

const EXACT_CHART_WORDS: &[&str] = &[
    "roc",
    "quantile",
    "scatter",
    "saçılım",
    "serpilme",
    "histogram",
    "eğri",
    "kutu grafiği",
    "box plot",
    "regresyon",
    "bar chart",
    "pasta grafiği",
    "çizgi grafiği",
];

const QUALITATIVE_CHART_WORDS: &[&str] = &[
    "küme",
    "cluster",
    "kmeans",
    "k-means",
    "dbscan",
    "hiyerarşik",
    "hierarchical",
    "ayrık",
    "ayrik",
    "örtüş",
    "ortus",
    "merkez",
    "centroid",
    "şematik",
    "schematic",
    "demetleme",
    "clustering",
];

const DIAGRAM_WORDS: &[&str] = &[
    "akış",
    "akis",
    "süreç",
    "surec",
    "şema",
    "sema",
    "diyagram",
    "diagram",
    "ağaç",
    "agac",
    "soyağacı",
    "soyagaci",
    "mimari",
    "sınıflandırma",
    "siniflandirma",
    "döngü",
    "dongu",
    "durum",
    "ilişki",
    "iliski",
    "erd",
    "uml",
    "adım",
    "adim",
    "katman",
    "model oluşturma",
    "model olusturma",
    "sözdizim",
    "sozdizim",
    "syntax",
    "gramer",
    "grammar",
];

const PHOTO_WORDS: &[&str] = &[
    "fotoğraf",
    "fotograf",
    "photo",
    "karikatür",
    "karikatur",
    "cover",
    "kapak",
    "portre",
    "screenshot",
    "ekran görüntüsü",
    "ekran goruntusu",
];

fn has(text: &str, words: &[&str]) -> bool {
    words.iter().any(|word| text.contains(word))
}

pub fn classify(input: ClassifyInput<'_>) -> ClassifyResult {
    let title = input.title.to_lowercase();
    let body = input.body.to_lowercase();
    let page_text = input.page_text.to_lowercase();
    let page_chart = has(&page_text, CHART_WORDS);
    let page_exact = has(&page_text, EXACT_CHART_WORDS);
    let chart_title = has(&title, CHART_WORDS);
    let exact_chart = has(&title, EXACT_CHART_WORDS)
        || has(&body, EXACT_CHART_WORDS)
        || (input.native_image && input.text_inside_chars < 50 && page_exact);
    let qualitative_chart =
        has(&title, QUALITATIVE_CHART_WORDS) || has(&body, QUALITATIVE_CHART_WORDS);
    let chart = chart_title
        || has(&body, CHART_WORDS)
        || (input.native_image && input.text_inside_chars < 50 && page_chart);
    let diagram_title = has(&title, DIAGRAM_WORDS);
    let diagram = diagram_title
        || (input.vector_primitives >= 20 && has(&body, DIAGRAM_WORDS) && !exact_chart);
    let photo = has(&title, PHOTO_WORDS) || has(&body, PHOTO_WORDS);

    let text_heavy = input.text_inside_chars > 300
        && input.vector_primitives < 300
        && !input.table
        && !input.native_image;
    let content_class = if input.table {
        "table"
    } else if text_heavy {
        "text"
    } else if diagram_title {
        "diagram"
    } else if chart {
        "chart"
    } else if diagram {
        "diagram"
    } else if photo || input.native_image {
        "photo"
    } else if input.vector_primitives >= 8 {
        "diagram"
    } else {
        "text"
    };

    let mut reasons = Vec::new();
    if input.table {
        reasons.push("table block with extractable structure".to_string());
    }
    if chart {
        reasons.push("chart keyword in heading/context".to_string());
    }
    if exact_chart {
        reasons.push("exact chart keyword (values/axes matter)".to_string());
    }
    if qualitative_chart {
        reasons.push("qualitative cluster/schematic keyword".to_string());
    }
    if diagram {
        reasons.push("diagram keyword in heading/context".to_string());
    }
    if input.vector_primitives >= 8 {
        reasons.push(format!(
            "{} vector primitives inside the bbox",
            input.vector_primitives
        ));
    }

    let (relevance, relevance_confidence) = if input.decorative {
        ("decorative".to_string(), "high".to_string())
    } else if text_heavy {
        ("supporting".to_string(), "medium".to_string())
    } else if input.table || chart || diagram || input.low_text {
        ("core".to_string(), "high".to_string())
    } else if photo {
        ("supporting".to_string(), "medium".to_string())
    } else {
        ("supporting".to_string(), "low".to_string())
    };

    let (reconstruction, reconstruction_confidence) = if input.table {
        ("table".to_string(), "high".to_string())
    } else if text_heavy {
        ("text".to_string(), "high".to_string())
    } else if exact_chart {
        ("unique".to_string(), "high".to_string())
    } else if diagram_title {
        ("d2_concept".to_string(), "high".to_string())
    } else if chart && qualitative_chart {
        ("d2_schematic".to_string(), "medium".to_string())
    } else if chart {
        ("unique".to_string(), "medium".to_string())
    } else if diagram {
        ("d2_concept".to_string(), "high".to_string())
    } else if photo || input.native_image {
        ("unique".to_string(), "medium".to_string())
    } else if input.vector_primitives >= 8 {
        // Dense vector content with no chart/diagram keyword is too ambiguous
        // to redraw safely; keep the source.
        ("unique".to_string(), "low".to_string())
    } else if input.text_inside_chars > 200 {
        ("text".to_string(), "medium".to_string())
    } else {
        ("unique".to_string(), "low".to_string())
    };

    ClassifyResult {
        content_class: content_class.to_string(),
        relevance,
        relevance_confidence,
        reconstruction,
        reconstruction_confidence,
        reasons,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input<'a>(title: &'a str, body: &'a str) -> ClassifyInput<'a> {
        ClassifyInput {
            title,
            body,
            page_text: title,
            table: false,
            native_image: false,
            vector_primitives: 20,
            low_text: false,
            text_inside_chars: 0,
            decorative: false,
        }
    }

    #[test]
    fn a_syntax_graph_is_a_concept_diagram() {
        let result = classify(input("Sözdizim Grafikleri", "ifade terim faktör"));
        assert_eq!(result.content_class, "diagram");
        assert_eq!(result.reconstruction, "d2_concept");
        assert_eq!(result.relevance, "core");
    }

    #[test]
    fn an_exact_roc_chart_stays_unique() {
        let result = classify(input("ROC Eğrisinin Çizilmesi", "TPR FPR eşik"));
        assert_eq!(result.reconstruction, "unique");
    }

    #[test]
    fn a_qualitative_cluster_chart_can_be_schematic() {
        let result = classify(input("Kümeleme", "kümeler ayrık ve örtüşüyor"));
        assert_eq!(result.reconstruction, "d2_schematic");
        assert_eq!(result.relevance, "core");
    }

    #[test]
    fn a_text_heavy_slide_is_not_a_visual() {
        let mut value = input("Model Parametrelerini Belirleme", "model oluşturma");
        value.text_inside_chars = 700;
        value.vector_primitives = 20;
        let result = classify(value);
        assert_eq!(result.content_class, "text");
        assert_eq!(result.reconstruction, "text");
    }

    #[test]
    fn a_decorative_image_is_capped() {
        let mut value = input("Kapak", "");
        value.decorative = true;
        let result = classify(value);
        assert_eq!(result.relevance, "decorative");
    }
}
