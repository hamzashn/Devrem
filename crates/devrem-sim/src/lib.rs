//! Devrem simülasyon motoru.
//!
//! ```
//! use devrem_core::{ComponentKind, DocBuilder, Point};
//! use devrem_sim::Simulator;
//!
//! // 10 V kaynak ve iki 1 kΩ dirençten oluşan gerilim bölücü
//! let mut b = DocBuilder::new();
//! let v = b.add(ComponentKind::DcVoltage, Point(0, 0), 0, &[("voltage", "10")]);
//! let r1 = b.add(ComponentKind::Resistor, Point(4, -2), 0, &[("resistance", "1k")]);
//! let r2 = b.add(ComponentKind::Resistor, Point(8, 0), 1, &[("resistance", "1k")]);
//! let g = b.add(ComponentKind::Ground, Point(0, 4), 0, &[]);
//! b.wire(b.pin(v, 0), b.pin(r1, 0));
//! b.wire(b.pin(r1, 1), b.pin(r2, 0));
//! b.wire(b.pin(r2, 1), b.pin(v, 1));
//! b.wire(b.pin(v, 1), b.pin(g, 0));
//!
//! let mut sim = Simulator::new(&b.finish()).unwrap();
//! let frame = sim.dc_operating_point().unwrap();
//! let topo = sim.topology();
//! let mid = topo.elements[2].nodes[0]; // R2'nin üst ucu
//! assert!((frame.node_voltages[mid] - 5.0).abs() < 1e-6);
//! ```

mod lu;
mod simulator;
pub mod spice;

pub use simulator::{
    AcProbe, AcResult, DcSweepResult, Frame, Method, Probe, ProbeKind, Simulator, SweepProbe,
    Topology, TopologyElement, TopologySegment,
};

use devrem_core::{CircuitError, Document};

/// Motor sürümü.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Sıfır başlangıç koşullarıyla `stop` saniyelik geçici durum analizi yapar ve
/// her `step` saniyede bir sonuç döner.
pub fn transient(doc: &Document, stop: f64, step: f64) -> Result<Vec<Frame>, CircuitError> {
    let mut sim = Simulator::new(doc)?;
    let h = step.min(sim.max_step());
    let mut frames = vec![sim.frame()];
    let mut next_output = step;
    while sim.time() < stop - 1e-15 {
        let dt = h.min(next_output - sim.time()).max(1e-15);
        sim.step(dt)?;
        if sim.time() >= next_output - 1e-15 {
            frames.push(sim.frame());
            next_output += step;
        }
    }
    Ok(frames)
}
