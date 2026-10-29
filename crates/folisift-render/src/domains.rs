//! Domain profiles are prompt/validation metadata, not layouts.

use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainProfile {
    pub id: String,
    pub name: String,
    pub emphasis: String,
    pub structure_default: String,
    pub checks: Vec<String>,
}

const BUILTINS: [(&str, &str); 8] = [
    ("general", include_str!("../../../domains/general.json")),
    ("cs", include_str!("../../../domains/cs.json")),
    ("math", include_str!("../../../domains/math.json")),
    ("medicine", include_str!("../../../domains/medicine.json")),
    ("law", include_str!("../../../domains/law.json")),
    (
        "humanities",
        include_str!("../../../domains/humanities.json"),
    ),
    ("business", include_str!("../../../domains/business.json")),
    ("language", include_str!("../../../domains/language.json")),
];

pub fn list(workspace_root: &Path) -> Result<Vec<DomainProfile>> {
    let mut profiles = Vec::new();
    for (id, json) in BUILTINS {
        profiles.push(parse(id, json)?);
    }
    let dir = workspace_root.join("domains");
    if dir.is_dir() {
        for entry in std::fs::read_dir(&dir).map_err(|error| crate::error::Error::Io {
            path: dir.clone(),
            source: error,
        })? {
            let entry = entry.map_err(|error| crate::error::Error::Io {
                path: dir.clone(),
                source: error,
            })?;
            let path = entry.path();
            if path.extension().and_then(|value| value.to_str()) == Some("json")
                && let Some(id) = path.file_stem().and_then(|value| value.to_str())
            {
                let json =
                    std::fs::read_to_string(&path).map_err(|error| crate::error::Error::Io {
                        path: path.clone(),
                        source: error,
                    })?;
                let profile = parse(id, &json)?;
                if let Some(existing) = profiles.iter_mut().find(|item| item.id == id) {
                    *existing = profile;
                } else {
                    profiles.push(profile);
                }
            }
        }
    }
    profiles.sort_by(|left, right| left.id.cmp(&right.id));
    Ok(profiles)
}

pub fn load(workspace_root: &Path, id: &str) -> Result<DomainProfile> {
    let custom = workspace_root.join("domains").join(format!("{id}.json"));
    if custom.is_file() {
        let json = std::fs::read_to_string(&custom).map_err(|error| crate::error::Error::Io {
            path: custom.clone(),
            source: error,
        })?;
        return parse(id, &json);
    }
    BUILTINS
        .iter()
        .find(|(builtin, _)| *builtin == id)
        .ok_or_else(|| Error::Style {
            id: id.to_string(),
            message: "unknown domain profile".to_string(),
        })
        .and_then(|(_, json)| parse(id, json))
}

fn parse(id: &str, json: &str) -> Result<DomainProfile> {
    serde_json::from_str(json).map_err(|error| Error::Style {
        id: id.to_string(),
        message: format!("domain profile: {error}"),
    })
}
