//! `examples/` klasöründeki örnek devrelerin açılıp çözülebildiğini doğrular.

use devrem_core::{Document, Point};
use devrem_sim::Simulator;
use devrem_sim::Topology;
use std::path::PathBuf;

fn examples_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples")
}

fn load(name: &str) -> Document {
    let path = examples_dir().join(name);
    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path:?}: {e}"));
    Document::from_json(&text).unwrap_or_else(|e| panic!("{name}: {e}"))
}

#[test]
fn all_examples_simulate() {
    let mut count = 0;
    for entry in std::fs::read_dir(examples_dir()).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().is_none_or(|e| e != "dvrm") {
            continue;
        }
        let name = path.file_name().unwrap().to_string_lossy().into_owned();
        let doc = load(&name);
        let mut sim = Simulator::new(&doc).unwrap_or_else(|e| panic!("{name}: {e}"));
        for _ in 0..200 {
            sim.step(5e-3).unwrap_or_else(|e| panic!("{name}: {e}"));
        }
        count += 1;
    }
    assert!(count >= 12, "örnek sayısı: {count}");
}

#[test]
fn led_example_current() {
    let doc = load("led.dvrm");
    let mut sim = Simulator::new(&doc).unwrap();
    let frame = sim.dc_operating_point().unwrap();
    let topo = sim.topology();
    let am = topo.elements.iter().position(|e| e.id == "AM1").unwrap();
    let current = frame.pin_currents[am][0];
    // (9 V − ~1.85 V) / 470 Ω ≈ 15 mA
    assert!((0.014..0.0158).contains(&current), "LED akımı {current}");
}

#[test]
fn divider_example_reading() {
    let doc = load("gerilim-bolucu.dvrm");
    let mut sim = Simulator::new(&doc).unwrap();
    let frame = sim.dc_operating_point().unwrap();
    let topo = sim.topology();
    let vm = topo.elements.iter().find(|e| e.id == "VM1").unwrap();
    let reading = frame.node_voltages[vm.nodes[0]] - frame.node_voltages[vm.nodes[1]];
    assert!((reading - 4.0).abs() < 1e-6, "voltmetre {reading}");
}

#[test]
fn rc_example_charges_when_switch_closes() {
    let doc = load("rc-sarj.dvrm");
    let mut sim = Simulator::new(&doc).unwrap();
    let topo = sim.topology();
    let c = topo.elements.iter().find(|e| e.id == "C1").unwrap();
    let vc = |sim: &Simulator| {
        let f = sim.frame();
        f.node_voltages[c.nodes[0]] - f.node_voltages[c.nodes[1]]
    };
    for _ in 0..100 {
        sim.step(1e-2).unwrap();
    }
    assert!(vc(&sim).abs() < 1e-3, "anahtar açıkken şarj olmamalı");
    sim.set_toggle("S1", true);
    for _ in 0..100 {
        sim.step(1e-2).unwrap(); // 1 s = 1τ
    }
    let expected = 5.0 * (1.0 - (-1.0f64).exp());
    assert!((vc(&sim) - expected).abs() < 0.02, "v_C = {}", vc(&sim));
}

fn node_v(topo: &Topology, f: &devrem_sim::Frame, id: &str, pin: usize) -> f64 {
    let e = topo.elements.iter().find(|e| e.id == id).unwrap();
    f.node_voltages[e.nodes[pin]]
}

#[test]
fn rc_low_pass_example_corner() {
    let doc = load("rc-alcak-geciren.dvrm");
    let mut sim = Simulator::new(&doc).unwrap();
    let result = sim.ac_analysis(&doc.simulation.ac).unwrap();
    let k = result
        .frequencies
        .iter()
        .position(|f| (f - 1000.0).abs() < 1e-6)
        .unwrap();
    let db = result.probes[0].magnitude_db[k];
    assert!((db + 3.0).abs() < 0.05, "{db} dB");
}

#[test]
fn transistor_switch_example() {
    let doc = load("transistor-anahtar.dvrm");
    let mut sim = Simulator::new(&doc).unwrap();
    let topo = sim.topology();
    let led = topo.elements.iter().position(|e| e.id == "LED1").unwrap();
    let off = sim.dc_operating_point().unwrap().pin_currents[led][0];
    assert!(off.abs() < 1e-6, "kapalıyken LED akımı {off}");
    sim.set_toggle("S1", true);
    let on = sim.dc_operating_point().unwrap().pin_currents[led][0];
    assert!((0.01..0.014).contains(&on), "açıkken LED akımı {on}");
}

#[test]
fn inverting_amplifier_example() {
    let doc = load("evirici-yukselteci.dvrm");
    let mut sim = Simulator::new(&doc).unwrap();
    let topo = sim.topology();
    for _ in 0..250 {
        sim.step(1e-3).unwrap(); // t = 0.25 s: giriş tepe değerde (0.5 V)
    }
    let f = sim.frame();
    let vin = node_v(&topo, &f, "VM1", 0) - node_v(&topo, &f, "VM1", 1);
    let vout = node_v(&topo, &f, "VM2", 0) - node_v(&topo, &f, "VM2", 1);
    assert!((vin - 0.5).abs() < 1e-3, "giriş {vin}");
    assert!((vout + 10.0 * vin).abs() < 1e-2, "çıkış {vout}");
}

#[test]
fn counter_example_counts() {
    let doc = load("sayici.dvrm");
    let mut sim = Simulator::new(&doc).unwrap();
    let topo = sim.topology();
    let mut last = [false; 2];
    let mut transitions = [0; 2];
    for _ in 0..2100 {
        sim.step(1e-3).unwrap();
        let f = sim.frame();
        for (k, id) in ["OUT1", "OUT2"].into_iter().enumerate() {
            let q = node_v(&topo, &f, id, 0) > 2.5;
            if q != last[k] {
                transitions[k] += 1;
                last[k] = q;
            }
        }
    }
    // 2 Hz saat t = 0'da yüksek başlar; 2.1 s içinde yükselen kenarlar 0.5, 1.0,
    // 1.5 ve 2.0 s'de → Q1 4 kez, Q2 (Q1'in düşen kenarlarında) 2 kez değişir.
    // Açılışta sahte kenar olmamalı.
    assert_eq!(transitions, [4, 2]);
    let _ = Point(0, 0);
}

#[test]
fn flasher_555_blinks() {
    // f = 1.44 / ((1k + 2·47k) · 10 µF) ≈ 1.52 Hz
    let doc = load("555-flasor.dvrm");
    let mut sim = Simulator::new(&doc).unwrap();
    let topo = sim.topology();
    let led = topo.elements.iter().position(|e| e.id == "LED1").unwrap();
    let mut last = false;
    let mut turn_ons = 0;
    for _ in 0..4000 {
        sim.step(1e-3).unwrap();
        let on = sim.frame().pin_currents[led][0] > 1e-3;
        if on && !last {
            turn_ons += 1;
        }
        last = on;
    }
    // 4 s içinde ~6 yanma (ilk çevrim daha uzun sürer)
    assert!((5..=7).contains(&turn_ons), "{turn_ons} kez yandı");
}

#[test]
fn hex_counter_shows_digit() {
    let doc = load("hex-sayac.dvrm");
    let mut sim = Simulator::new(&doc).unwrap();
    let topo = sim.topology();
    // 2 Hz saat t = 0'da yüksek başlar: 0.5 … 3.0 s arasında 6 yükselen kenar → 6
    for _ in 0..3200 {
        sim.step(1e-3).unwrap();
    }
    let f = sim.frame();
    let lit: Vec<bool> = (0..7).map(|s| node_v(&topo, &f, "DS1", s) > 2.5).collect();
    // 6 → a, c, d, e, f, g (b sönük)
    assert_eq!(lit, [true, false, true, true, true, true, true]);
}

#[test]
fn dimmer_example_current() {
    let doc = load("isik-ayari.dvrm");
    let mut sim = Simulator::new(&doc).unwrap();
    let topo = sim.topology();
    let lamp = topo.elements.iter().position(|e| e.id == "LP1").unwrap();
    // Reosta: %20 · 500 Ω = 100 Ω, lamba 144 Ω → 12 / 244 ≈ 49 mA
    let i = sim.dc_operating_point().unwrap().pin_currents[lamp][0];
    assert!((i - 12.0 / 244.0).abs() < 1e-4, "{i}");
    sim.set_control("RV1", 0.0);
    let i = sim.dc_operating_point().unwrap().pin_currents[lamp][0];
    assert!((i - 12.0 / 144.0).abs() < 1e-3, "{i}");
}

#[test]
fn zener_example_regulates() {
    let doc = load("zener-regulator.dvrm");
    let mut sim = Simulator::new(&doc).unwrap();
    let topo = sim.topology();
    let f = sim.dc_operating_point().unwrap();
    let v = node_v(&topo, &f, "VM1", 0) - node_v(&topo, &f, "VM1", 1);
    assert!((v - 5.1).abs() < 0.08, "{v}");
}
