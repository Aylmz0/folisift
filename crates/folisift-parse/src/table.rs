//! Draft table extraction from native PDF text boxes.
//!
//! This is intentionally a draft: a host/LLM must review cells before they
//! become guide content. No OCR and no ML model are used.

use std::path::Path;

use serde::Serialize;

use crate::error::{Error, Result};
use crate::pdfium;

#[derive(Debug, Clone, Serialize)]
pub struct ExtractedTable {
    pub page: usize,
    pub header: Vec<String>,
    pub rows: Vec<Vec<String>>,
    pub confidence: f64,
    pub draft: bool,
    pub bbox: BBox,
}

#[derive(Debug, Clone, Serialize)]
pub struct BBox {
    pub left: f64,
    pub top: f64,
    pub right: f64,
    pub bottom: f64,
}

type PositionedChar = (f64, f64, f64, f64, char);
type Line = Vec<PositionedChar>;

#[derive(Debug, Clone)]
struct Cell {
    x: f64,
    text: String,
}

/// Extract candidate tables from one page. Returns an empty vector when the
/// page has no repeating multi-column row structure.
pub fn extract_tables(path: &Path, page_number: usize) -> Result<Vec<ExtractedTable>> {
    pdfium::init(None)?;
    let path_text = path.to_str().ok_or_else(|| Error::Parse {
        path: path.to_path_buf(),
        message: "PDF path is not valid UTF-8".to_string(),
    })?;
    let library = liteparse_pdfium::Library::try_init().map_err(|error| Error::Pdfium {
        message: error.to_string(),
    })?;
    let document = library
        .load_document(path_text, None)
        .map_err(|error| Error::Parse {
            path: path.to_path_buf(),
            message: error.to_string(),
        })?;
    let index = i32::try_from(page_number.saturating_sub(1)).map_err(|_| Error::PageSelection {
        value: page_number.to_string(),
    })?;
    if index >= document.page_count() {
        return Err(Error::PageSelection {
            value: page_number.to_string(),
        });
    }
    let page = document.page(index).map_err(|error| Error::Parse {
        path: path.to_path_buf(),
        message: error.to_string(),
    })?;
    let text = page.text().map_err(|error| Error::Parse {
        path: path.to_path_buf(),
        message: error.to_string(),
    })?;

    let mut chars = Vec::new();
    for i in 0..text.char_count() {
        let Some(ch) = text.char_at(i) else { continue };
        let rect = text
            .rect(i)
            .map(|rect| (rect.left, rect.top, rect.right, rect.bottom))
            .or_else(|| {
                ch.char_box()
                    .map(|bbox| (bbox.left, bbox.top, bbox.right, bbox.bottom))
            })
            .or_else(|| {
                ch.origin().map(|(x, y)| {
                    let size = ch.font_size();
                    (x, y, x + size * 0.6, y + size)
                })
            });
        let Some((left, top, right, bottom)) = rect else {
            continue;
        };
        if let Some(value) = char::from_u32(ch.unicode())
            && !value.is_control()
        {
            chars.push((left, top, right, bottom, value));
        }
    }
    if chars.is_empty() {
        return Ok(Vec::new());
    }
    let (left, top, right, bottom) = chars.iter().fold(
        (f64::MAX, f64::MAX, f64::MIN, f64::MIN),
        |(left, top, right, bottom), (x1, y1, x2, y2, _)| {
            (left.min(*x1), top.min(*y1), right.max(*x2), bottom.max(*y2))
        },
    );

    let bbox = BBox {
        left,
        top,
        right,
        bottom,
    };
    Ok(extract_from_positions(page_number, chars, bbox))
}

fn extract_from_positions(
    page_number: usize,
    mut chars: Vec<PositionedChar>,
    bbox: BBox,
) -> Vec<ExtractedTable> {
    // Group characters into visual lines by their top coordinate.
    chars.sort_by(|a, b| a.1.total_cmp(&b.1).then(a.0.total_cmp(&b.0)));
    let mut lines: Vec<Line> = Vec::new();
    for ch in chars {
        if let Some(line) = lines.last_mut()
            && (line[0].1 - ch.1).abs() <= 2.5
        {
            line.push(ch);
            continue;
        }
        lines.push(vec![ch]);
    }

    let mut parsed = Vec::new();
    for mut line in lines {
        line.sort_by(|a, b| a.0.total_cmp(&b.0));
        let mut cells: Vec<Cell> = Vec::new();
        let mut current = String::new();
        let mut current_x = line.first().map_or(0.0, |ch| ch.0);
        let mut last_right = current_x;
        for (left, _top, right, _bottom, value) in line {
            if !current.is_empty() && left - last_right > 30.0 {
                cells.push(Cell {
                    x: current_x,
                    text: current.trim().to_string(),
                });
                current.clear();
                current_x = left;
            }
            current.push(value);
            last_right = right;
        }
        if !current.trim().is_empty() {
            cells.push(Cell {
                x: current_x,
                text: current.trim().to_string(),
            });
        }
        cells.retain(|cell| !cell.text.is_empty());
        if cells.len() >= 2 {
            parsed.push(cells);
        }
    }

    if parsed.len() < 3 {
        return Vec::new();
    }
    let mut groups: std::collections::HashMap<usize, Vec<Vec<Cell>>> =
        std::collections::HashMap::new();
    for row in parsed {
        groups.entry(row.len()).or_default().push(row);
    }
    let candidate = groups
        .into_iter()
        .filter(|(columns, rows)| {
            *columns >= 3
                && *columns <= 8
                && rows.len() >= 3
                && rows
                    .iter()
                    .flat_map(|row| row.iter().map(|cell| cell.text.chars().count()))
                    .sum::<usize>()
                    / rows.iter().map(Vec::len).sum::<usize>()
                    < 55
        })
        .max_by_key(|(_, rows)| rows.len());
    let Some((columns, mut rows)) = candidate else {
        return Vec::new();
    };
    // The same column must start at roughly the same x on every row.
    let aligned = (0..columns).all(|column| {
        let positions = rows
            .iter()
            .filter_map(|row| row.get(column).map(|cell| cell.x))
            .collect::<Vec<_>>();
        let (min, max) = positions
            .iter()
            .copied()
            .fold((f64::MAX, f64::MIN), |(min, max), x| {
                (min.min(x), max.max(x))
            });
        max - min <= 18.0
    });
    if !aligned || rows.len() < 3 {
        return Vec::new();
    }
    rows.truncate(20);
    let table_rows = rows
        .into_iter()
        .map(|row| {
            (0..columns)
                .map(|index| {
                    row.get(index)
                        .map(|cell| cell.text.clone())
                        .unwrap_or_default()
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    let confidence = (0.55 + (table_rows.len() as f64 / 6.0).min(1.0) * 0.4).min(0.95);
    vec![ExtractedTable {
        page: page_number,
        header: table_rows[0].clone(),
        rows: table_rows[1..].to_vec(),
        confidence: (confidence * 100.0).round() / 100.0,
        draft: true,
        bbox,
    }]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn grid(rows: &[&[&str]]) -> Vec<PositionedChar> {
        let mut chars = Vec::new();
        for (row_index, row) in rows.iter().enumerate() {
            let y = 100.0 + row_index as f64 * 20.0;
            for (column, cell) in row.iter().enumerate() {
                let x = 10.0 + column as f64 * 200.0;
                for (offset, value) in cell.chars().enumerate() {
                    let left = x + offset as f64 * 5.0;
                    chars.push((left, y, left + 5.0, y + 10.0, value));
                }
            }
        }
        chars
    }

    fn bbox() -> BBox {
        BBox {
            left: 0.0,
            top: 90.0,
            right: 500.0,
            bottom: 180.0,
        }
    }

    #[test]
    fn recovers_a_simple_grid() {
        let chars = grid(&[
            &["Evre", "Girdi", "Çıktı"],
            &["Sözcük", "Karakter", "Token"],
            &["Sözdizim", "Token", "Ağaç"],
            &["Kod", "Ağaç", "Bayt"],
        ]);
        let tables = extract_from_positions(3, chars, bbox());
        assert_eq!(tables.len(), 1);
        let table = &tables[0];
        assert_eq!(table.header, vec!["Evre", "Girdi", "Çıktı"]);
        assert_eq!(table.rows.len(), 3);
        assert_eq!(table.rows[0], vec!["Sözcük", "Karakter", "Token"]);
        assert!(table.draft);
        assert!(table.confidence > 0.5);
    }

    #[test]
    fn rejects_misaligned_columns() {
        let mut chars = grid(&[&["A", "B", "C"], &["D", "E", "F"], &["G", "H", "I"]]);
        for ch in &mut chars {
            if ch.4 == 'E' {
                ch.0 += 60.0;
                ch.2 += 60.0;
            }
        }
        assert!(extract_from_positions(1, chars, bbox()).is_empty());
    }

    #[test]
    fn rejects_a_single_row() {
        let chars = grid(&[&["A", "B", "C"]]);
        assert!(extract_from_positions(1, chars, bbox()).is_empty());
    }
}
