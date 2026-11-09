//! Programla devre kurmak için yardımcı (testler ve örnekler için).

use crate::document::{Component, Document, Wire};
use crate::geometry::Point;
use crate::library::ComponentKind;
use std::collections::HashMap;

/// Adım adım `Document` oluşturur.
///
/// ```
/// use devrem_core::{ComponentKind, DocBuilder, Point};
///
/// let mut b = DocBuilder::new();
/// let v = b.add(ComponentKind::DcVoltage, Point(0, 0), 0, &[("voltage", "5")]);
/// let r = b.add(ComponentKind::Resistor, Point(4, -2), 0, &[("resistance", "1k")]);
/// let g = b.add(ComponentKind::Ground, Point(0, 4), 0, &[]);
/// b.wire(b.pin(v, 0), b.pin(r, 0));
/// b.wire(b.pin(v, 1), b.pin(g, 0));
/// let doc = b.finish();
/// assert_eq!(doc.components.len(), 3);
/// ```
pub struct DocBuilder {
    doc: Document,
    counters: HashMap<&'static str, usize>,
}

impl Default for DocBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl DocBuilder {
    pub fn new() -> Self {
        Self {
            doc: Document::new("Test"),
            counters: HashMap::new(),
        }
    }

    /// Bileşen ekler ve sıra numarasını döner.
    pub fn add(
        &mut self,
        kind: ComponentKind,
        pos: Point,
        rot: u8,
        props: &[(&str, &str)],
    ) -> usize {
        let n = self.counters.entry(kind.prefix()).or_insert(0);
        *n += 1;
        self.doc.components.push(Component {
            id: format!("{}{}", kind.prefix(), n),
            kind,
            pos,
            rot,
            props: props
                .iter()
                .map(|(k, v)| ((*k).to_owned(), (*v).into()))
                .collect(),
        });
        self.doc.components.len() - 1
    }

    /// Bileşen özelliğini değiştirir.
    pub fn set_prop(&mut self, component: usize, key: &str, value: serde_json::Value) {
        self.doc.components[component]
            .props
            .insert(key.to_owned(), value);
    }

    /// Bileşenin `index` numaralı pininin konumu.
    pub fn pin(&self, component: usize, index: usize) -> Point {
        self.doc.components[component]
            .pin_positions()
            .nth(index)
            .expect("pin indeksi geçersiz")
    }

    /// Bileşen kimliği (R1, C1 ...).
    pub fn id(&self, component: usize) -> &str {
        &self.doc.components[component].id
    }

    /// İki nokta arasına tel çeker.
    pub fn wire(&mut self, a: Point, b: Point) {
        let id = format!("W{}", self.doc.wires.len() + 1);
        self.doc.wires.push(Wire { id, a, b });
    }

    pub fn finish(self) -> Document {
        self.doc
    }
}
