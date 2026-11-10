//! Transformatör ve optokuplör.

use devrem_core::{AcSettings, ComponentKind as K, DocBuilder, Document, Point};
use devrem_sim::{Frame, Simulator, Topology, spice::to_spice};

struct Circuit {
    b: DocBuilder,
    next: i32,
}

impl Circuit {
    fn new() -> Self {
        Self {
            b: DocBuilder::new(),
            next: 0,
        }
    }

    fn add(&mut self, kind: K, props: &[(&str, &str)]) -> usize {
        self.next += 1;
        let n = self.next;
        self.b.add(kind, Point(n * 37, (n * n * 13) % 97), 0, props)
    }

    fn wire(&mut self, a: (usize, usize), b: (usize, usize)) {
        let (pa, pb) = (self.b.pin(a.0, a.1), self.b.pin(b.0, b.1));
        self.b.wire(pa, pb);
    }

    fn ground(&mut self, pin: (usize, usize)) {
        let g = self.add(K::Ground, &[]);
        self.wire(pin, (g, 0));
    }

    fn finish(self) -> Document {
        self.b.finish()
    }
}

fn element(topo: &Topology, id: &str) -> usize {
    topo.elements.iter().position(|e| e.id == id).unwrap()
}

fn v(topo: &Topology, f: &Frame, id: &str, pin: usize) -> f64 {
    f.node_voltages[topo.elements[element(topo, id)].nodes[pin]]
}

fn i(topo: &Topology, f: &Frame, id: &str, pin: usize) -> f64 {
    f.pin_currents[element(topo, id)][pin]
}

/// 10:1 düşürücü transformatör; ikincil 1 kΩ yüklü ve toprağa bağlı değil.
fn step_down(source: (K, &[(&str, &str)])) -> Document {
    let mut c = Circuit::new();
    let src = c.add(source.0, source.1);
    let t = c.add(
        K::Transformer,
        &[("primary", "1"), ("ratio", "0.1"), ("coupling", "0.999")],
    );
    let load = c.add(K::Resistor, &[("resistance", "1k")]);
    let vm = c.add(K::Voltmeter, &[]);
    c.ground((src, 1));
    c.wire((src, 0), (t, 0));
    c.ground((t, 1));
    c.wire((t, 2), (load, 0));
    c.wire((t, 3), (load, 1));
    c.wire((vm, 0), (t, 2));
    c.wire((vm, 1), (t, 3));
    c.finish()
}

#[test]
fn transformer_ac_ratio_and_isolation() {
    let doc = step_down((
        K::AcVoltage,
        &[
            ("amplitude", "10"),
            ("frequency", "100"),
            ("offset", "0"),
            ("phase", "0"),
        ],
    ));
    let mut sim = Simulator::new(&doc).unwrap();
    let result = sim
        .ac_analysis(&AcSettings {
            start: 10.0,
            stop: 1000.0,
            points_per_decade: 10,
        })
        .unwrap();
    let k = result
        .frequencies
        .iter()
        .position(|f| (f - 100.0).abs() < 1e-6)
        .unwrap();
    let probe = &result.probes[0];
    let gain = 10f64.powf(probe.magnitude_db[k] / 20.0);
    // V2 / V1 = k · n = 0.999 · 0.1 (yük etkisi ihmal edilebilir)
    assert!((gain - 0.0999).abs() < 0.0999 * 0.01, "kazanç {gain}");
    assert!(
        probe.phase_deg[k].abs() < 2.0,
        "faz {}°",
        probe.phase_deg[k]
    );
}

#[test]
fn transformer_transient_amplitude() {
    let doc = step_down((
        K::AcVoltage,
        &[
            ("amplitude", "10"),
            ("frequency", "50"),
            ("offset", "0"),
            ("phase", "0"),
        ],
    ));
    let mut sim = Simulator::new(&doc).unwrap();
    let topo = sim.topology();
    let mut peak: f64 = 0.0;
    for n in 0..2000 {
        sim.step(1e-4).unwrap();
        if n >= 1800 {
            // Son çevrim (0.18–0.2 s)
            let f = sim.frame();
            peak = peak.max((v(&topo, &f, "VM1", 0) - v(&topo, &f, "VM1", 1)).abs());
        }
    }
    assert!((peak - 0.999).abs() < 0.02, "ikincil tepe {peak} V");
}

#[test]
fn transformer_blocks_dc() {
    let mut c = Circuit::new();
    let src = c.add(K::DcVoltage, &[("voltage", "10")]);
    let r = c.add(K::Resistor, &[("resistance", "100")]);
    let t = c.add(K::Transformer, &[]);
    let load = c.add(K::Resistor, &[("resistance", "1k")]);
    c.ground((src, 1));
    c.wire((src, 0), (r, 0));
    c.wire((r, 1), (t, 0));
    c.ground((t, 1));
    c.wire((t, 2), (load, 0));
    c.ground((load, 1));
    c.ground((t, 3));
    let mut sim = Simulator::new(&c.finish()).unwrap();
    let topo = sim.topology();
    // τ = L1 / R = 10 ms; 100 ms sonra birincil akımı oturmuş olmalı
    for _ in 0..1000 {
        sim.step(1e-4).unwrap();
    }
    let f = sim.frame();
    assert!(
        v(&topo, &f, "T1", 2).abs() < 1e-3,
        "ikincil {}",
        v(&topo, &f, "T1", 2)
    );
    assert!(
        (i(&topo, &f, "T1", 0) - 0.1).abs() < 1e-3,
        "birincil akımı 10 V / 100 Ω"
    );
}

/// LED: 5 V → 330 Ω → optokuplör; çıkış: 5 V → yük → kollektör, emetör toprakta.
fn opto_circuit(input: &str, load: &str) -> Document {
    let mut c = Circuit::new();
    let vin = c.add(K::DcVoltage, &[("voltage", input)]);
    let rin = c.add(K::Resistor, &[("resistance", "330")]);
    let ok = c.add(K::Optocoupler, &[("ctr", "100")]);
    let vcc = c.add(K::DcVoltage, &[("voltage", "5")]);
    let rl = c.add(K::Resistor, &[("resistance", load)]);
    c.ground((vin, 1));
    c.wire((vin, 0), (rin, 0));
    c.wire((rin, 1), (ok, 0));
    c.ground((ok, 1));
    c.ground((vcc, 1));
    c.wire((vcc, 0), (rl, 0));
    c.wire((rl, 1), (ok, 2));
    c.ground((ok, 3));
    c.finish()
}

#[test]
fn optocoupler_switches_output() {
    let doc = opto_circuit("5", "1k");
    let mut sim = Simulator::new(&doc).unwrap();
    let topo = sim.topology();
    let f = sim.dc_operating_point().unwrap();
    let id = i(&topo, &f, "OK1", 0);
    assert!((0.010..0.013).contains(&id), "LED akımı {id}");
    let vce = v(&topo, &f, "OK1", 2);
    assert!(vce < 0.3, "doymada Vce = {vce}");

    let doc = opto_circuit("0", "1k");
    let mut sim = Simulator::new(&doc).unwrap();
    let topo = sim.topology();
    let f = sim.dc_operating_point().unwrap();
    assert!(
        v(&topo, &f, "OK1", 2) > 4.99,
        "LED sönükken çıkış açık devre"
    );
}

#[test]
fn optocoupler_linear_region_follows_ctr() {
    let doc = opto_circuit("5", "100");
    let mut sim = Simulator::new(&doc).unwrap();
    let topo = sim.topology();
    let f = sim.dc_operating_point().unwrap();
    let id = i(&topo, &f, "OK1", 0);
    let ic = i(&topo, &f, "OK1", 2);
    // Vce ≈ 5 − 100 Ω · 11.5 mA ≫ VCE_SAT: Ic = CTR · I_LED
    assert!((ic / id - 1.0).abs() < 0.01, "Ic/Id = {}", ic / id);
    // KCL: emetörden çıkan akım kollektörden girene eşit
    assert!((i(&topo, &f, "OK1", 3) + ic).abs() < 1e-12);
}

#[test]
fn spice_export_of_coupled_devices() {
    let tx = to_spice(&step_down((K::DcVoltage, &[("voltage", "1")]))).unwrap();
    assert!(tx.contains("LT1P "), "{tx}");
    assert!(tx.contains("LT1S "), "{tx}");
    assert!(tx.contains("KT1 LT1P LT1S 0.999"), "{tx}");

    let ok = to_spice(&opto_circuit("5", "1k")).unwrap();
    assert!(ok.contains("FOK1 "), "{ok}");
    assert!(ok.contains(".model DOPTO_IR D("), "{ok}");
}
