use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum ChartMode {
    NATAL,
    EVENT,
    HORARY,
    COMPOSITE,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HouseSystem {
    #[serde(rename = "Placidus")]
    Placidus,
    #[serde(rename = "Whole Sign")]
    WholeSign,
    #[serde(rename = "Campanus")]
    Campanus,
    #[serde(rename = "Koch")]
    Koch,
    #[serde(rename = "Equal")]
    Equal,
    #[serde(rename = "Regiomontanus")]
    Regiomontanus,
    #[serde(rename = "Vehlow")]
    Vehlow,
    #[serde(rename = "Porphyry")]
    Porphyry,
    #[serde(rename = "Alcabitius")]
    Alcabitius,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum ZodiacType {
    Tropical,
    Sidereal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EngineType {
    #[serde(rename = "swisseph")]
    Swisseph,
    #[serde(rename = "jyotish")]
    Jyotish,
    #[serde(rename = "jpl")]
    Jpl,
    #[serde(rename = "custom")]
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Location {
    pub name: String,
    pub latitude: f64,
    pub longitude: f64,
    pub timezone: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartSubject {
    pub id: String,
    pub name: String,
    #[serde(deserialize_with = "deserialize_datetime", serialize_with = "serialize_datetime")]
    pub event_time: Option<chrono::DateTime<chrono::Utc>>,
    pub location: Location,
}

fn deserialize_datetime<'de, D>(deserializer: D) -> Result<Option<chrono::DateTime<chrono::Utc>>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::Deserialize;
    let s: Option<String> = Option::deserialize(deserializer)?;
    match s {
        Some(ref s) if !s.is_empty() => {
            // Try multiple datetime formats
            let formats = [
                "%Y-%m-%d %H:%M:%S",
                "%Y-%m-%dT%H:%M:%S",
                "%Y-%m-%dT%H:%M:%SZ",
                "%Y-%m-%d %H:%M",
                "%Y-%m-%d",
            ];
            
            for format in &formats {
                if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(s, format) {
                    return Ok(Some(dt.and_utc()));
                }
            }
            
            // Try parsing as ISO8601
            if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(s) {
                return Ok(Some(dt.with_timezone(&chrono::Utc)));
            }
            
            Ok(None)
        }
        _ => Ok(None),
    }
}

fn serialize_datetime<S>(dt: &Option<chrono::DateTime<chrono::Utc>>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    match dt {
        Some(dt) => serializer.serialize_str(&dt.format("%Y-%m-%d %H:%M:%S").to_string()),
        None => serializer.serialize_none(),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartConfig {
    pub mode: ChartMode,
    #[serde(default)]
    pub house_system: Option<HouseSystem>,
    pub zodiac_type: ZodiacType,
    #[serde(default)]
    pub included_points: Vec<String>,
    #[serde(default)]
    pub aspect_orbs: HashMap<String, f64>,
    #[serde(default)]
    pub display_style: String,
    #[serde(default)]
    pub color_theme: String,
    pub override_ephemeris: Option<String>,
    pub model: Option<String>,
    pub engine: Option<EngineType>,
    pub ayanamsa: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartInstance {
    pub id: String,
    pub subject: ChartSubject,
    pub config: ChartConfig,
    #[serde(default)]
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceDefaults {
    pub ephemeris_engine: Option<EngineType>,
    pub ephemeris_backend: Option<String>,
    pub default_location: Option<Location>,
    pub language: Option<String>,
    pub theme: Option<String>,
    pub default_house_system: Option<HouseSystem>,
    #[serde(default)]
    pub default_bodies: Option<Vec<String>>,
    #[serde(default)]
    pub default_aspects: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceManifest {
    pub owner: String,
    pub active_model: Option<String>,
    #[serde(default)]
    pub aspects: Vec<String>,
    pub default: WorkspaceDefaults,
    #[serde(default)]
    pub chart_presets: Vec<String>, // File paths
    #[serde(default)]
    pub subjects: Vec<String>, // File paths
    #[serde(default)]
    pub charts: Vec<String>, // File paths
    #[serde(default)]
    pub layouts: Vec<String>, // File paths
    #[serde(default)]
    pub annotations: Vec<String>, // File paths
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartSummary {
    pub id: String,
    pub name: String,
    pub chart_type: String,
    pub date_time: String,
    pub location: String,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceInfo {
    pub path: String,
    pub owner: String,
    pub active_model: Option<String>,
    pub charts: Vec<ChartSummary>,
}
