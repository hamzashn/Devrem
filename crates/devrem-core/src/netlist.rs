//! Şemadan netlist üretimi.
//!
//! Teller ve pinler ızgara noktalarında birleşir. Bir telin ortasına denk gelen
//! pin veya tel ucu (T bağlantısı) teli o noktada böler. Birbirine bağlı
//! noktalar aynı elektriksel düğümü oluşturur; toprağa bağlı düğüm 0'dır.
//!
//! Netlist ayrıca tel parçalarındaki akımları hesaplamak için bir yayılan
//! ağaç planı tutar: bileşen pinlerinden giren/çıkan akımlar ağaç boyunca
//! toplanarak her parçanın akımı bulunur.

use crate::document::Document;
use crate::error::CircuitError;
use crate::geometry::{Point, sort_along, strictly_inside_segment};
use crate::library::ComponentKind;
use std::collections::{HashMap, VecDeque};

/// Simülasyona giren bir bileşen (toprak hariç).
#[derive(Debug, Clone)]
pub struct NetElement {
    /// `Document::components` içindeki sırası.
    pub component: usize,
    /// Her pinin bağlı olduğu düğüm.
    pub nodes: Vec<usize>,
    /// Her pinin nokta indeksi (akım dağıtımı için).
    pin_points: Vec<usize>,
}

/// Bir telin bölünmüş parçası.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Segment {
    /// `Document::wires` içindeki sırası.
    pub wire: usize,
    pub a: Point,
    pub b: Point,
    pub node: usize,
}

#[derive(Debug, Clone)]
struct FlowEdge {
    child: usize,
    parent: usize,
    segment: usize,
    child_is_a: bool,
}

#[derive(Debug, Clone)]
pub struct Netlist {
    /// Toprak (0) dahil düğüm sayısı.
    pub node_count: usize,
    pub elements: Vec<NetElement>,
    pub segments: Vec<Segment>,
    point_count: usize,
    /// Yapraktan köke doğru sıralı ağaç kenarları.
    flow: Vec<FlowEdge>,
}

struct UnionFind(Vec<usize>);

impl UnionFind {
    fn find(&mut self, mut i: usize) -> usize {
        while self.0[i] != i {
            self.0[i] = self.0[self.0[i]];
            i = self.0[i];
        }
        i
    }
    fn union(&mut self, a: usize, b: usize) {
        let (ra, rb) = (self.find(a), self.find(b));
        if ra != rb {
            self.0[rb] = ra;
        }
    }
}

#[derive(Default)]
struct Points {
    list: Vec<Point>,
    index: HashMap<Point, usize>,
}

impl Points {
    fn intern(&mut self, p: Point) -> usize {
        *self.index.entry(p).or_insert_with(|| {
            self.list.push(p);
            self.list.len() - 1
        })
    }
}

impl Netlist {
    pub fn build(doc: &Document) -> Result<Self, CircuitError> {
        let mut points = Points::default();

        let comp_pins: Vec<Vec<usize>> = doc
            .components
            .iter()
            .map(|c| c.pin_positions().map(|p| points.intern(p)).collect())
            .collect();
        for w in &doc.wires {
            points.intern(w.a);
            points.intern(w.b);
        }

        // Telleri üzerlerindeki bağlantı noktalarından böl.
        let anchors = points.list.clone();
        let mut raw_segments: Vec<(usize, usize, usize)> = Vec::new();
        for (wi, w) in doc.wires.iter().enumerate() {
            if w.a == w.b {
                continue;
            }
            let mut inner: Vec<Point> = anchors
                .iter()
                .copied()
                .filter(|&p| strictly_inside_segment(p, w.a, w.b))
                .collect();
            sort_along(&mut inner, w.a);
            let mut prev = points.index[&w.a];
            for p in inner.into_iter().chain([w.b]) {
                let next = points.index[&p];
                raw_segments.push((wi, prev, next));
                prev = next;
            }
        }

        let point_count = points.list.len();
        let mut uf = UnionFind((0..point_count).collect());
        for &(_, a, b) in &raw_segments {
            uf.union(a, b);
        }

        let ground_points: Vec<usize> = doc
            .components
            .iter()
            .zip(&comp_pins)
            .filter(|(c, _)| c.kind == ComponentKind::Ground)
            .flat_map(|(_, pins)| pins.iter().copied())
            .collect();
        // Akım kaynağı/kuyusu sayılan noktalar: toprak + beslemeli çıkışlar.
        let root_points: Vec<usize> = doc
            .components
            .iter()
            .zip(&comp_pins)
            .flat_map(|(c, pins)| c.kind.supply_pins().iter().map(|&i| pins[i]))
            .collect();

        if doc
            .components
            .iter()
            .all(|c| c.kind == ComponentKind::Ground)
        {
            return Err(CircuitError::EmptyCircuit);
        }
        // Op-amp ve lojik çıkışlar toprağa göre sürülür; böyle bir bileşen varsa
        // toprak sembolü olmadan da referans (düğüm 0) tanımlıdır.
        if ground_points.is_empty() && root_points.is_empty() {
            return Err(CircuitError::NoGround);
        }

        // Düğüm numaralama: toprağa bağlı kökler 0, diğerleri sırayla.
        let mut root_node: HashMap<usize, usize> = HashMap::new();
        for &g in &ground_points {
            let root = uf.find(g);
            root_node.insert(root, 0);
        }
        let mut node_count = 1;
        let mut point_node = vec![0; point_count];
        for (p, node) in point_node.iter_mut().enumerate() {
            let root = uf.find(p);
            *node = *root_node.entry(root).or_insert_with(|| {
                node_count += 1;
                node_count - 1
            });
        }

        let elements = doc
            .components
            .iter()
            .zip(&comp_pins)
            .enumerate()
            .filter(|(_, (c, _))| c.kind != ComponentKind::Ground)
            .map(|(i, (_, pins))| NetElement {
                component: i,
                nodes: pins.iter().map(|&p| point_node[p]).collect(),
                pin_points: pins.clone(),
            })
            .collect();

        let segments: Vec<Segment> = raw_segments
            .iter()
            .map(|&(wire, a, b)| Segment {
                wire,
                a: points.list[a],
                b: points.list[b],
                node: point_node[a],
            })
            .collect();

        let flow = Self::plan_flow(point_count, &raw_segments, &root_points);

        Ok(Self {
            node_count,
            elements,
            segments,
            point_count,
            flow,
        })
    }

    /// Her bağlı tel grubu için yayılan ağaç kurar. Grupta toprak veya
    /// beslemeli bir çıkış varsa kök olarak o seçilir; böylece gruptaki akım
    /// dengesizliği (görünmeyen besleme akımı) köke toplanır.
    fn plan_flow(
        point_count: usize,
        segments: &[(usize, usize, usize)],
        root_points: &[usize],
    ) -> Vec<FlowEdge> {
        let mut adj: Vec<Vec<(usize, usize)>> = vec![Vec::new(); point_count];
        for (si, &(_, a, b)) in segments.iter().enumerate() {
            adj[a].push((si, b));
            adj[b].push((si, a));
        }
        let mut visited = vec![false; point_count];
        let mut order: Vec<FlowEdge> = Vec::new();
        let roots = root_points.iter().copied().chain(0..point_count);
        for root in roots {
            if visited[root] {
                continue;
            }
            visited[root] = true;
            let mut queue = VecDeque::from([root]);
            while let Some(v) = queue.pop_front() {
                for &(si, w) in &adj[v] {
                    if !visited[w] {
                        visited[w] = true;
                        order.push(FlowEdge {
                            child: w,
                            parent: v,
                            segment: si,
                            child_is_a: segments[si].1 == w,
                        });
                        queue.push_back(w);
                    }
                }
            }
        }
        order.reverse();
        order
    }

    /// Tel parçalarındaki akımları hesaplar.
    ///
    /// `pin_currents[i][k]`, `elements[i]` bileşeninin `k` numaralı pininden
    /// bileşene giren akımdır. Sonuç, her parça için `a`'dan `b`'ye akan akımdır.
    pub fn segment_currents(&self, pin_currents: &[Vec<f64>]) -> Vec<f64> {
        let mut acc = vec![0.0; self.point_count];
        for (e, currents) in self.elements.iter().zip(pin_currents) {
            for (&p, &i) in e.pin_points.iter().zip(currents) {
                acc[p] -= i;
            }
        }
        let mut out = vec![0.0; self.segments.len()];
        for edge in &self.flow {
            let s = acc[edge.child];
            acc[edge.parent] += s;
            out[edge.segment] = if edge.child_is_a { s } else { -s };
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::builder::DocBuilder;

    #[test]
    fn requires_ground() {
        let mut b = DocBuilder::new();
        b.add(ComponentKind::Resistor, Point(0, 0), 0, &[]);
        assert_eq!(
            Netlist::build(&b.finish()).unwrap_err(),
            CircuitError::NoGround
        );
    }

    #[test]
    fn logic_circuit_needs_no_ground_symbol() {
        let mut b = DocBuilder::new();
        let input = b.add(ComponentKind::LogicInput, Point(0, 0), 0, &[]);
        let probe = b.add(ComponentKind::LogicProbe, Point(10, 0), 0, &[]);
        b.wire(b.pin(input, 0), b.pin(probe, 0));
        let net = Netlist::build(&b.finish()).unwrap();
        assert_eq!(net.node_count, 2);
    }

    #[test]
    fn empty_circuit() {
        let mut b = DocBuilder::new();
        b.add(ComponentKind::Ground, Point(0, 0), 0, &[]);
        assert_eq!(
            Netlist::build(&b.finish()).unwrap_err(),
            CircuitError::EmptyCircuit
        );
    }

    #[test]
    fn t_junction_splits_wire() {
        // Yatay bir telin ortasına dikey bir tel değiyor.
        let mut b = DocBuilder::new();
        let r = b.add(ComponentKind::Resistor, Point(10, 10), 0, &[]);
        let g = b.add(ComponentKind::Ground, Point(20, 20), 0, &[]);
        b.wire(Point(0, 0), Point(8, 0));
        b.wire(Point(4, 0), Point(4, 10));
        b.wire(Point(4, 10), b.pin(r, 0));
        b.wire(b.pin(r, 1), Point(20, 10));
        b.wire(Point(20, 10), b.pin(g, 0));
        let doc = b.finish();
        let net = Netlist::build(&doc).unwrap();

        // Uzun tel 2 parçaya bölünmeli: 1 + 1 + 1 + 1 + 1 = 6 parça
        assert_eq!(net.segments.len(), 6);
        // Toprak + üst düğüm = 2 düğüm
        assert_eq!(net.node_count, 2);
        assert_eq!(net.elements[0].nodes, vec![1, 0]);
    }
}
