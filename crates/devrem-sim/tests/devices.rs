//! Faz 2 bileşenleri ve analizleri: transistörler, op-amp, AC analizi,
//! DC tarama ve dijital lojik.

use devrem_core::{
    AcSettings, CircuitError, ComponentKind as K, DcSweepSettings, DocBuilder, Document, Point,
};
use devrem_sim::{Frame, Simulator, Topology};

/// Bileşenleri birbirinden uzak, çapraz tellerin başka pinlerden geçmeyeceği
/// konumlara yerleştiren yardımcı.
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

    /// Bir pini toprağa bağlar (her bağlantı için ayrı toprak sembolü).
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

fn dc(doc: &Document) -> (Topology, Frame) {
    let mut sim = Simulator::new(doc).unwrap();
    let frame = sim.dc_operating_point().unwrap();
    (sim.topology(), frame)
}

#[test]
fn npn_common_emitter_bias() {
    // Vcc = 10 V, RB = 100 kΩ, RC = 500 Ω, β = 100
    let mut c = Circuit::new();
    let vcc = c.add(K::DcVoltage, &[("voltage", "10")]);
    let rb = c.add(K::Resistor, &[("resistance", "100k")]);
    let rc = c.add(K::Resistor, &[("resistance", "500")]);
    let q = c.add(K::Npn, &[("beta", "100")]);
    c.ground((vcc, 1));
    c.wire((vcc, 0), (rb, 0));
    c.wire((vcc, 0), (rc, 0));
    c.wire((rb, 1), (q, 1));
    c.wire((rc, 1), (q, 0));
    c.ground((q, 2));
    let (topo, f) = dc(&c.finish());

    let ib = i(&topo, &f, "Q1", 1);
    let ic = i(&topo, &f, "Q1", 0);
    let ie = i(&topo, &f, "Q1", 2);
    assert!((ic / ib - 100.0).abs() < 1.0, "β = {}", ic / ib);
    assert!((ic + ib + ie).abs() < 1e-12, "KCL");
    let vbe = v(&topo, &f, "Q1", 1) - v(&topo, &f, "Q1", 2);
    assert!((0.55..0.75).contains(&vbe), "Vbe = {vbe}");
    let expected_ic = (10.0 - vbe) / 100e3 * 100.0;
    let vce = v(&topo, &f, "Q1", 0);
    assert!(
        (vce - (10.0 - expected_ic * 500.0)).abs() < 0.05,
        "Vce = {vce}"
    );
}

#[test]
fn pnp_high_side() {
    // Emetör 10 V'ta, baz 100 kΩ ile toprağa, kollektör 500 Ω ile toprağa.
    let mut c = Circuit::new();
    let vcc = c.add(K::DcVoltage, &[("voltage", "10")]);
    let rb = c.add(K::Resistor, &[("resistance", "100k")]);
    let rc = c.add(K::Resistor, &[("resistance", "500")]);
    let q = c.add(K::Pnp, &[("beta", "100")]);
    c.ground((vcc, 1));
    c.wire((vcc, 0), (q, 2));
    c.wire((q, 1), (rb, 0));
    c.ground((rb, 1));
    c.wire((q, 0), (rc, 0));
    c.ground((rc, 1));
    let (topo, f) = dc(&c.finish());

    let ic = i(&topo, &f, "Q1", 0);
    assert!(ic < 0.0, "PNP kollektöründen akım çıkar: {ic}");
    let vc = v(&topo, &f, "Q1", 0);
    assert!((vc - (-ic) * 500.0).abs() < 1e-6);
    assert!((4.0..5.0).contains(&vc), "Vc = {vc}");
}

fn mosfet_switch(kind: K, gate: &str) -> (Topology, Frame) {
    let mut c = Circuit::new();
    let vdd = c.add(K::DcVoltage, &[("voltage", "10")]);
    let vg = c.add(K::DcVoltage, &[("voltage", gate)]);
    let r = c.add(K::Resistor, &[("resistance", "1k")]);
    let m = c.add(kind, &[("vth", "2"), ("k", "100m")]);
    c.ground((vdd, 1));
    c.ground((vg, 1));
    c.wire((vg, 0), (m, 1));
    if kind == K::Nmos {
        c.wire((vdd, 0), (r, 0));
        c.wire((r, 1), (m, 0));
        c.ground((m, 2));
    } else {
        c.wire((vdd, 0), (m, 2));
        c.wire((m, 0), (r, 0));
        c.ground((r, 1));
    }
    dc(&c.finish())
}

#[test]
fn nmos_switch() {
    let (topo, on) = mosfet_switch(K::Nmos, "5");
    let vds = v(&topo, &on, "M1", 0);
    assert!(vds < 0.1, "açık NMOS Vds = {vds}");
    let id = i(&topo, &on, "M1", 0);
    assert!((id - (10.0 - vds) / 1e3).abs() < 1e-9);

    let (topo, off) = mosfet_switch(K::Nmos, "0");
    assert!(i(&topo, &off, "M1", 0).abs() < 1e-9);
}

#[test]
fn pmos_switch() {
    let (topo, on) = mosfet_switch(K::Pmos, "0");
    assert!(v(&topo, &on, "M1", 0) > 9.9);
    let (topo, off) = mosfet_switch(K::Pmos, "10");
    assert!(v(&topo, &off, "M1", 0) < 1e-3);
}

#[test]
fn opamp_inverting_amplifier() {
    // Kazanç = −Rf / Rin = −10
    let mut c = Circuit::new();
    let vin = c.add(K::DcVoltage, &[("voltage", "0.5")]);
    let rin = c.add(K::Resistor, &[("resistance", "1k")]);
    let rf = c.add(K::Resistor, &[("resistance", "10k")]);
    let op = c.add(K::OpAmp, &[]);
    c.ground((vin, 1));
    c.wire((vin, 0), (rin, 0));
    c.wire((rin, 1), (op, 0));
    c.wire((rf, 0), (op, 0));
    c.wire((rf, 1), (op, 2));
    c.ground((op, 1));
    let (topo, f) = dc(&c.finish());
    let out = v(&topo, &f, "OP1", 2);
    assert!((out + 5.0).abs() < 1e-3, "çıkış {out}");
}

#[test]
fn opamp_comparator_saturates() {
    let mut c = Circuit::new();
    let vin = c.add(K::DcVoltage, &[("voltage", "1")]);
    let op = c.add(K::OpAmp, &[("vsat", "12")]);
    c.ground((vin, 1));
    c.wire((vin, 0), (op, 1));
    c.ground((op, 0));
    let (topo, f) = dc(&c.finish());
    assert!((v(&topo, &f, "OP1", 2) - 12.0).abs() < 1e-6);
}

fn rc_low_pass() -> Document {
    // fc = 1 / (2π · 1 kΩ · 159.155 nF) ≈ 1 kHz
    let mut c = Circuit::new();
    let src = c.add(
        K::AcVoltage,
        &[
            ("amplitude", "1"),
            ("frequency", "1k"),
            ("offset", "0"),
            ("phase", "0"),
        ],
    );
    let r = c.add(K::Resistor, &[("resistance", "1k")]);
    let cap = c.add(K::Capacitor, &[("capacitance", "159.155n")]);
    let vm = c.add(K::Voltmeter, &[]);
    c.ground((src, 1));
    c.wire((src, 0), (r, 0));
    c.wire((r, 1), (cap, 0));
    c.ground((cap, 1));
    c.wire((vm, 0), (cap, 0));
    c.ground((vm, 1));
    c.finish()
}

#[test]
fn ac_rc_low_pass_corner() {
    let mut sim = Simulator::new(&rc_low_pass()).unwrap();
    let result = sim
        .ac_analysis(&AcSettings {
            start: 10.0,
            stop: 100e3,
            points_per_decade: 20,
        })
        .unwrap();
    assert_eq!(result.reference, "V1");
    let k = result
        .frequencies
        .iter()
        .position(|f| (f - 1000.0).abs() < 1e-6)
        .expect("1 kHz noktası");
    let probe = &result.probes[0];
    assert!(
        (probe.magnitude_db[k] + 3.0103).abs() < 0.01,
        "{} dB",
        probe.magnitude_db[k]
    );
    assert!(
        (probe.phase_deg[k] + 45.0).abs() < 0.05,
        "{}°",
        probe.phase_deg[k]
    );
    // Düşük frekansta kazanç 0 dB, yüksekte −20 dB/dekad
    assert!(probe.magnitude_db[0].abs() < 0.01);
    let last = probe.magnitude_db.last().unwrap();
    assert!((last + 40.0).abs() < 0.1, "100 kHz: {last} dB");
}

#[test]
fn ac_requires_source_and_probe() {
    let mut c = Circuit::new();
    let v = c.add(K::DcVoltage, &[]);
    let r = c.add(K::Resistor, &[]);
    let vm = c.add(K::Voltmeter, &[]);
    c.ground((v, 1));
    c.wire((v, 0), (r, 0));
    c.ground((r, 1));
    c.wire((vm, 0), (r, 0));
    c.ground((vm, 1));
    let mut sim = Simulator::new(&c.finish()).unwrap();
    assert_eq!(
        sim.ac_analysis(&AcSettings::default()).unwrap_err(),
        CircuitError::NoAcSource
    );
}

#[test]
fn dc_sweep_diode_curve() {
    let mut c = Circuit::new();
    let v = c.add(K::DcVoltage, &[("voltage", "0")]);
    let r = c.add(K::Resistor, &[("resistance", "10")]);
    let am = c.add(K::Ammeter, &[]);
    let d = c.add(K::Diode, &[]);
    c.ground((v, 1));
    c.wire((v, 0), (r, 0));
    c.wire((r, 1), (am, 0));
    c.wire((am, 1), (d, 0));
    c.ground((d, 1));
    let mut sim = Simulator::new(&c.finish()).unwrap();
    let result = sim
        .dc_sweep(&DcSweepSettings {
            source: "V1".into(),
            start: 0.0,
            stop: 2.0,
            steps: 40,
        })
        .unwrap();
    assert_eq!(result.values.len(), 41);
    let current = &result.probes[0].data;
    assert!(current[0].abs() < 1e-9);
    assert!(
        current.windows(2).all(|w| w[1] >= w[0]),
        "akım monoton artmalı"
    );
    // 2 V'ta: (2 − ~0.75) / 10 Ω ≈ 0.12 A
    assert!((0.1..0.15).contains(current.last().unwrap()));

    let err = sim
        .dc_sweep(&DcSweepSettings {
            source: "R1".into(),
            ..DcSweepSettings::default()
        })
        .unwrap_err();
    assert!(matches!(err, CircuitError::InvalidSweepSource { .. }));
}

#[test]
fn logic_gates_follow_inputs() {
    let mut c = Circuit::new();
    let a = c.add(K::LogicInput, &[]);
    let b = c.add(K::LogicInput, &[]);
    let gate = c.add(K::And, &[]);
    let probe = c.add(K::LogicProbe, &[]);
    c.wire((a, 0), (gate, 0));
    c.wire((b, 0), (gate, 1));
    c.wire((gate, 2), (probe, 0));
    // Devrede bir toprak bulunmalı; lojik çıkışlar toprağa göre sürülür.
    c.add(K::Ground, &[]);
    let doc = c.finish();
    let mut sim = Simulator::new(&doc).unwrap();
    let topo = sim.topology();
    let out = |sim: &mut Simulator| {
        let f = sim.dc_operating_point().unwrap();
        v(&topo, &f, "OUT1", 0)
    };
    assert!(out(&mut sim) < 0.1);
    sim.set_toggle("IN1", true);
    assert!(out(&mut sim) < 0.1);
    sim.set_toggle("IN2", true);
    assert!((out(&mut sim) - 5.0).abs() < 1e-3);
}

#[test]
fn t_flip_flop_divides_clock() {
    let mut c = Circuit::new();
    let clk = c.add(K::Clock, &[("frequency", "10")]);
    let high = c.add(K::LogicInput, &[]);
    let ff = c.add(K::TFlipFlop, &[]);
    let probe = c.add(K::LogicProbe, &[]);
    c.add(K::Ground, &[]);
    c.wire((high, 0), (ff, 0));
    c.wire((clk, 0), (ff, 1));
    c.wire((ff, 2), (probe, 0));
    let mut sim = Simulator::new(&c.finish()).unwrap();
    sim.set_toggle("IN1", true);
    let topo = sim.topology();

    let mut last = false;
    let mut transitions = 0;
    for _ in 0..1000 {
        sim.step(1e-3).unwrap();
        let q = v(&topo, &sim.frame(), "OUT1", 0) > 2.5;
        if q != last {
            transitions += 1;
            last = q;
        }
    }
    // 1 s'de 10 yükselen kenar → Q 10 kez değişir (5 Hz kare dalga)
    assert!((9..=11).contains(&transitions), "{transitions} geçiş");
}

#[test]
fn bjt_wire_currents_balance() {
    // Transistörlü devrede her tel parçasının akımı, bağlı pin akımlarıyla tutarlı olmalı.
    let mut c = Circuit::new();
    let vcc = c.add(K::DcVoltage, &[("voltage", "10")]);
    let rb = c.add(K::Resistor, &[("resistance", "100k")]);
    let rc = c.add(K::Resistor, &[("resistance", "500")]);
    let q = c.add(K::Npn, &[]);
    c.ground((vcc, 1));
    c.wire((vcc, 0), (rb, 0));
    c.wire((vcc, 0), (rc, 0));
    c.wire((rb, 1), (q, 1));
    c.wire((rc, 1), (q, 0));
    c.ground((q, 2));
    let doc = c.finish();
    let (topo, f) = dc(&doc);
    // W4: RB → baz teli (Ib), W5: RC → kollektör teli (Ic)
    let seg = |wire: &str| {
        let k = topo
            .segments
            .iter()
            .position(|s| s.wire_id == wire)
            .unwrap();
        f.segment_currents[k].abs()
    };
    assert!((seg("W4") - i(&topo, &f, "Q1", 1)).abs() < 1e-12);
    assert!((seg("W5") - i(&topo, &f, "Q1", 0)).abs() < 1e-9);
}
