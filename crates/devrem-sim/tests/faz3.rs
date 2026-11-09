//! Faz 3 bileşenleri: potansiyometre, zener, lamba, iki yollu anahtar, kare
//! dalga kaynağı, 555 zamanlayıcı, onaltılık çözücü ve 7 segment gösterge.

use devrem_core::{ComponentKind as K, DocBuilder, Document, Point};
use devrem_sim::{Frame, Simulator, Topology};

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

#[test]
fn potentiometer_divider_and_live_control() {
    let mut c = Circuit::new();
    let src = c.add(K::DcVoltage, &[("voltage", "10")]);
    let pot = c.add(K::Potentiometer, &[("resistance", "10k")]);
    c.b.set_prop(pot, "position", 30.into());
    c.ground((src, 1));
    c.wire((src, 0), (pot, 0));
    c.ground((pot, 1));
    let mut sim = Simulator::new(&c.finish()).unwrap();
    let topo = sim.topology();
    let f = sim.dc_operating_point().unwrap();
    // A–orta uç = %30 → orta uç gerilimi 10 · 0.7 = 7 V
    assert!((v(&topo, &f, "RV1", 2) - 7.0).abs() < 1e-6);

    assert!(sim.set_control("RV1", 0.8));
    let f = sim.dc_operating_point().unwrap();
    assert!((v(&topo, &f, "RV1", 2) - 2.0).abs() < 1e-6);
}

#[test]
fn zener_regulates() {
    let mut c = Circuit::new();
    let src = c.add(K::DcVoltage, &[("voltage", "12")]);
    let r = c.add(K::Resistor, &[("resistance", "1k")]);
    let z = c.add(K::Zener, &[("breakdown", "5.1")]);
    c.ground((src, 1));
    c.wire((src, 0), (r, 0));
    // Katot (pin 1) yukarıda, anot (pin 0) toprakta: ters kutuplama
    c.wire((r, 1), (z, 1));
    c.ground((z, 0));
    let mut sim = Simulator::new(&c.finish()).unwrap();
    let topo = sim.topology();
    let f = sim.dc_operating_point().unwrap();
    let vz = v(&topo, &f, "D1", 1);
    assert!((vz - 5.1).abs() < 0.05, "Vz = {vz}");
    // Kırılma akımı ≈ (12 − 5.1) / 1k
    assert!((i(&topo, &f, "D1", 1) - (12.0 - vz) / 1e3).abs() < 1e-9);
}

#[test]
fn lamp_resistance_from_rating() {
    let mut c = Circuit::new();
    let src = c.add(K::DcVoltage, &[("voltage", "12")]);
    let lamp = c.add(K::Lamp, &[("voltage", "12"), ("power", "1")]);
    c.ground((src, 1));
    c.wire((src, 0), (lamp, 0));
    c.ground((lamp, 1));
    let mut sim = Simulator::new(&c.finish()).unwrap();
    let topo = sim.topology();
    let f = sim.dc_operating_point().unwrap();
    // Anma geriliminde anma gücü: I = P / V
    assert!((i(&topo, &f, "LP1", 0) - 1.0 / 12.0).abs() < 1e-9);
}

#[test]
fn spdt_selects_output() {
    let mut c = Circuit::new();
    let src = c.add(K::DcVoltage, &[("voltage", "5")]);
    let sw = c.add(K::SpdtSwitch, &[]);
    let ra = c.add(K::Resistor, &[("resistance", "1k")]);
    let rb = c.add(K::Resistor, &[("resistance", "1k")]);
    c.ground((src, 1));
    c.wire((src, 0), (sw, 0));
    c.wire((sw, 1), (ra, 0));
    c.wire((sw, 2), (rb, 0));
    c.ground((ra, 1));
    c.ground((rb, 1));
    let mut sim = Simulator::new(&c.finish()).unwrap();
    let topo = sim.topology();
    let f = sim.dc_operating_point().unwrap();
    assert!((i(&topo, &f, "R1", 0) - 5e-3).abs() < 1e-6);
    assert!(i(&topo, &f, "R2", 0).abs() < 1e-9);
    sim.set_toggle("S1", true);
    let f = sim.dc_operating_point().unwrap();
    assert!(i(&topo, &f, "R1", 0).abs() < 1e-9);
    assert!((i(&topo, &f, "R2", 0) - 5e-3).abs() < 1e-6);
}

#[test]
fn pulse_source_square_wave() {
    let mut c = Circuit::new();
    let src = c.add(
        K::PulseVoltage,
        &[
            ("low", "0"),
            ("high", "5"),
            ("frequency", "10"),
            ("duty", "25"),
        ],
    );
    let r = c.add(K::Resistor, &[]);
    c.ground((src, 1));
    c.wire((src, 0), (r, 0));
    c.ground((r, 1));
    let mut sim = Simulator::new(&c.finish()).unwrap();
    let topo = sim.topology();
    let mut high = 0;
    for _ in 0..1000 {
        sim.step(1e-4).unwrap();
        if v(&topo, &sim.frame(), "R1", 0) > 2.5 {
            high += 1;
        }
    }
    // %25 doluluk oranı
    assert!((240..=260).contains(&high), "yüksek adım sayısı {high}");
}

#[test]
fn timer_555_astable_frequency() {
    // f = 1.44 / ((R1 + 2·R2)·C) = 1.44 / (21 kΩ · 10 µF) ≈ 6.86 Hz
    let mut c = Circuit::new();
    let vcc = c.add(K::DcVoltage, &[("voltage", "9")]);
    let u = c.add(K::Timer555, &[]);
    let r1 = c.add(K::Resistor, &[("resistance", "1k")]);
    let r2 = c.add(K::Resistor, &[("resistance", "10k")]);
    let cap = c.add(K::Capacitor, &[("capacitance", "10µ")]);
    let rl = c.add(K::Resistor, &[("resistance", "1k")]);
    c.ground((vcc, 1));
    c.wire((vcc, 0), (u, 7)); // VCC
    c.wire((vcc, 0), (u, 3)); // RESET
    c.ground((u, 0)); // GND
    c.wire((vcc, 0), (r1, 0));
    c.wire((r1, 1), (u, 6)); // DIS
    c.wire((u, 6), (r2, 0));
    c.wire((r2, 1), (u, 5)); // THR
    c.wire((u, 5), (u, 1)); // TRIG
    c.wire((u, 5), (cap, 0));
    c.ground((cap, 1));
    c.wire((u, 2), (rl, 0)); // OUT
    c.ground((rl, 1));
    let mut sim = Simulator::new(&c.finish()).unwrap();
    let topo = sim.topology();

    let h = 1e-4;
    let mut last = false;
    let mut rising = Vec::new();
    for n in 0..25_000 {
        sim.step(h).unwrap();
        let out = v(&topo, &sim.frame(), "U1", 2) > 4.5;
        if out && !last {
            rising.push(n as f64 * h);
        }
        last = out;
    }
    assert!(rising.len() >= 10, "{} kenar", rising.len());
    // İlk çevrim (kondansatör 0 V'tan dolar) hariç ortalama periyot
    let periods: Vec<f64> = rising.windows(2).skip(1).map(|w| w[1] - w[0]).collect();
    let mean = periods.iter().sum::<f64>() / periods.len() as f64;
    let expected = 0.693 * 21e3 * 10e-6;
    assert!(
        ((mean - expected) / expected).abs() < 0.05,
        "periyot {mean} s"
    );
}

#[test]
fn hex_decoder_drives_display() {
    let mut c = Circuit::new();
    let bits: Vec<usize> = (0..4).map(|_| c.add(K::LogicInput, &[])).collect();
    let dec = c.add(K::HexDecoder, &[]);
    let disp = c.add(K::SevenSegment, &[]);
    for (k, &b) in bits.iter().enumerate() {
        c.wire((b, 0), (dec, k));
    }
    for seg in 0..7 {
        c.wire((dec, 4 + seg), (disp, seg));
    }
    let mut sim = Simulator::new(&c.finish()).unwrap();
    let topo = sim.topology();
    // 5 = A + C
    sim.set_toggle("IN1", true);
    sim.set_toggle("IN3", true);
    let f = sim.dc_operating_point().unwrap();
    let lit: Vec<bool> = (0..7).map(|s| v(&topo, &f, "DS1", s) > 2.5).collect();
    // 5 → a, c, d, f, g
    assert_eq!(lit, [true, false, true, true, false, true, true]);
}
