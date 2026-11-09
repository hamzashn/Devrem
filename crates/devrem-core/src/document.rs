//! `.dvrm` proje dosyası.

use crate::geometry::Point;
use crate::library::ComponentKind;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Güncel `.dvrm` dosya formatı sürümü.
pub const FORMAT_VERSION: u32 = 1;

/// Bir Devrem projesi.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Document {
    pub format_version: u32,
    pub meta: DocumentMeta,
    #[serde(default)]
    pub components: Vec<Component>,
    #[serde(default)]
    pub wires: Vec<Wire>,
    #[serde(default)]
    pub simulation: SimulationSettings,
}

/// Proje hakkında bilgiler.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentMeta {
    pub name: String,
    pub created_with: String,
}

/// Şemaya yerleştirilmiş bir bileşen.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Component {
    /// Benzersiz isim (R1, C2 ...).
    pub id: String,
    pub kind: ComponentKind,
    /// Bileşen merkezinin ızgara konumu.
    pub pos: Point,
    /// 90° adımlarla saat yönünde dönüş (0–3).
    #[serde(default)]
    pub rot: u8,
    /// Bileşen özellikleri (ör. `"resistance": "4k7"`).
    #[serde(default)]
    pub props: BTreeMap<String, serde_json::Value>,
}

impl Component {
    /// Pinlerin ızgaradaki konumları.
    pub fn pin_positions(&self) -> impl Iterator<Item = Point> + '_ {
        self.kind
            .pins()
            .iter()
            .map(|p| self.pos + p.rotate(self.rot))
    }
}

/// İki ızgara noktası arasında düz bir tel.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Wire {
    pub id: String,
    pub a: Point,
    pub b: Point,
}

/// Simülasyon ayarları. Eksik alanlar varsayılan değerleri alır.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct SimulationSettings {
    /// Gerçek zamanlı modda 1 saniyede ilerleyen simülasyon süresi (s).
    pub time_scale: f64,
    pub ac: AcSettings,
    pub dc_sweep: DcSweepSettings,
}

impl Default for SimulationSettings {
    fn default() -> Self {
        Self {
            time_scale: 1.0,
            ac: AcSettings::default(),
            dc_sweep: DcSweepSettings::default(),
        }
    }
}

/// AC (frekans) analizi ayarları.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct AcSettings {
    /// Başlangıç frekansı (Hz).
    pub start: f64,
    /// Bitiş frekansı (Hz).
    pub stop: f64,
    /// Onlu (dekad) başına nokta sayısı.
    pub points_per_decade: u32,
}

impl Default for AcSettings {
    fn default() -> Self {
        Self {
            start: 1.0,
            stop: 1e6,
            points_per_decade: 20,
        }
    }
}

/// DC tarama ayarları.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct DcSweepSettings {
    /// Taranan DC gerilim/akım kaynağının kimliği.
    pub source: String,
    pub start: f64,
    pub stop: f64,
    /// Adım sayısı.
    pub steps: u32,
}

impl Default for DcSweepSettings {
    fn default() -> Self {
        Self {
            source: String::new(),
            start: 0.0,
            stop: 5.0,
            steps: 100,
        }
    }
}

/// Dosya okuma / yazma hataları.
#[derive(Debug, Clone, PartialEq, Serialize, thiserror::Error)]
#[serde(tag = "code", rename_all = "camelCase")]
pub enum DocumentError {
    #[error("dosya hatası: {message}")]
    Io { message: String },

    #[error("dosya okunamadı: {message}")]
    Parse { message: String },

    #[error("dosya daha yeni bir Devrem sürümüyle oluşturulmuş (format {version})")]
    NewerVersion { version: u32 },
}

impl Document {
    /// Verilen isimle boş bir proje oluşturur.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            format_version: FORMAT_VERSION,
            meta: DocumentMeta {
                name: name.into(),
                created_with: format!("Devrem {}", env!("CARGO_PKG_VERSION")),
            },
            components: Vec::new(),
            wires: Vec::new(),
            simulation: SimulationSettings::default(),
        }
    }

    /// JSON metninden proje okur ve sürümü denetler.
    pub fn from_json(json: &str) -> Result<Self, DocumentError> {
        let doc: Self = serde_json::from_str(json).map_err(|e| DocumentError::Parse {
            message: e.to_string(),
        })?;
        if doc.format_version > FORMAT_VERSION {
            return Err(DocumentError::NewerVersion {
                version: doc.format_version,
            });
        }
        Ok(doc)
    }

    /// Projeyi okunabilir JSON olarak yazar.
    pub fn to_json(&self) -> String {
        let mut json = serde_json::to_string_pretty(self).expect("Document her zaman serileşir");
        json.push('\n');
        json
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn json_round_trip() {
        let mut doc = Document::new("RC Filtre");
        doc.components.push(Component {
            id: "R1".into(),
            kind: ComponentKind::Resistor,
            pos: Point(4, 2),
            rot: 1,
            props: [("resistance".to_owned(), "4k7".into())].into(),
        });
        doc.wires.push(Wire {
            id: "W1".into(),
            a: Point(0, 0),
            b: Point(4, 0),
        });
        let json = doc.to_json();
        assert!(json.contains("\"formatVersion\": 1"));
        assert!(json.contains("\"kind\": \"resistor\""));
        assert_eq!(Document::from_json(&json).unwrap(), doc);
    }

    #[test]
    fn rejects_newer_format() {
        let json = r#"{"formatVersion": 99, "meta": {"name": "x", "createdWith": "y"}}"#;
        assert_eq!(
            Document::from_json(json),
            Err(DocumentError::NewerVersion { version: 99 })
        );
    }

    #[test]
    fn pin_positions_follow_rotation() {
        let c = Component {
            id: "R1".into(),
            kind: ComponentKind::Resistor,
            pos: Point(10, 10),
            rot: 1,
            props: BTreeMap::new(),
        };
        let pins: Vec<_> = c.pin_positions().collect();
        assert_eq!(pins, [Point(10, 8), Point(10, 12)]);
    }
}
