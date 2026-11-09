//! Analitik çözümü bilinen devrelerle motor doğrulaması.

use devrem_core::{CircuitError, ComponentKind as K, DocBuilder, Document, Point};
use devrem_sim::{Frame, Simulator, Topology, transient};

fn assert_close(actual: f64, expected: f64, rel: f64, what: &str) {
    let tol = rel * expected.abs().max(1e-12);
    assert!(
        (actual - expected).abs() <= tol,
        "{what}: {actual} ≠ {expected} (tolerans {tol})"
    );
}

fn element(topo: &Topology, id: &str) -> usize {
    topo.elements.iter().position(|e| e.id == id).unwrap()
}

fn pin_voltage(topo: &Topology, frame: &Frame, id: &str, pin: usize) -> f64 {
    frame.node_voltages[topo.elements[element(topo, id)].nodes[pin]]
}

fn current(topo: &Topology, frame: &Frame, id: &str) -> f64 {
    frame.pin_currents[element(topo, id)][0]
}

/// Kaynak (sol, dikey) → seri eleman(lar) → kaynağın eksi ucuna dönen basit
/// döngü. `parts` yatay olarak üst hatta sıralanır; son elemanın çıkışı
/// kaynağın eksi ucuna bağlanır.
fn series_loop(source: (K, &[(&str, &str)]), parts: &[(K, &[(&str, &str)])]) -> Document {
    let mut b = DocBuilder::new();
    let src = b.add(source.0, Point(0, 2), 0, source.1);
    let gnd = b.add(K::Ground, Point(0, 6), 0, &[]);
    b.wire(b.pin(src, 1), b.pin(gnd, 0));

    let mut prev = b.pin(src, 0);
    for (i, (kind, props)) in parts.iter().enumerate() {
        let x = 6 + 6 * i32::try_from(i).unwrap();
        let c = b.add(*kind, Point(x, 0), 0, props);
        b.wire(prev, b.pin(c, 0));
        prev = b.pin(c, 1);
    }
    // Dönüş hattı: son pinden aşağı, sonra kaynağın eksi ucuna.
    let bottom = Point(prev.0, 4);
    b.wire(prev, bottom);
    b.wire(bottom, b.pin(src, 1));
    b.finish()
}

#[test]
fn voltage_divider() {
    let mut b = DocBuilder::new();
    let v = b.add(K::DcVoltage, Point(0, 2), 0, &[("voltage", "12")]);
    let r1 = b.add(K::Resistor, Point(6, 0), 0, &[("resistance", "2k")]);
    let r2 = b.add(K::Resistor, Point(10, 4), 1, &[("resistance", "1k")]);
    let g = b.add(K::Ground, Point(0, 8), 0, &[]);
    b.wire(b.pin(v, 0), b.pin(r1, 0));
    b.wire(b.pin(r1, 1), Point(10, 0));
    b.wire(Point(10, 0), b.pin(r2, 0));
    b.wire(b.pin(r2, 1), Point(10, 8));
    b.wire(Point(10, 8), b.pin(g, 0));
    b.wire(b.pin(v, 1), b.pin(g, 0));
    let mut sim = Simulator::new(&b.finish()).unwrap();
    let frame = sim.dc_operating_point().unwrap();
    let topo = sim.topology();

    assert_close(pin_voltage(&topo, &frame, "R2", 0), 4.0, 1e-6, "V(R2)");
    assert_close(current(&topo, &frame, "R1"), 4e-3, 1e-6, "I(R1)");
    // Kaynaktan çıkan akım, pin 0'dan pin 1'e yönünde negatiftir.
    assert_close(current(&topo, &frame, "V1"), -4e-3, 1e-6, "I(V1)");
}

#[test]
fn wire_currents_follow_series_loop() {
    let doc = series_loop(
        (K::DcVoltage, &[("voltage", "5")]),
        &[(K::Resistor, &[("resistance", "1k")])],
    );
    let mut sim = Simulator::new(&doc).unwrap();
    let frame = sim.dc_operating_point().unwrap();
    let topo = sim.topology();
    for (seg, current) in topo.segments.iter().zip(&frame.segment_currents) {
        // W1 kaynağı toprağa bağlar; toprak bağlantısından akım geçmez.
        let expected = if seg.wire_id == "W1" { 0.0 } else { 5e-3 };
        assert!(
            (current.abs() - expected).abs() < 1e-8,
            "{}: {current}",
            seg.wire_id
        );
    }
}

#[test]
fn rc_charging() {
    // τ = RC = 1 ms; v_C(t) = 5·(1 − e^(−t/τ))
    let doc = series_loop(
        (K::DcVoltage, &[("voltage", "5")]),
        &[
            (K::Resistor, &[("resistance", "1k")]),
            (K::Capacitor, &[("capacitance", "1µ")]),
        ],
    );
    let frames = transient(&doc, 3e-3, 1e-5).unwrap();
    let topo = Simulator::new(&doc).unwrap().topology();
    for (t, expected) in [
        (1e-3, 5.0 * (1.0 - (-1.0f64).exp())),
        (3e-3, 5.0 * (1.0 - (-3.0f64).exp())),
    ] {
        let frame = frames
            .iter()
            .min_by(|a, b| (a.time - t).abs().total_cmp(&(b.time - t).abs()))
            .unwrap();
        let vc = pin_voltage(&topo, frame, "C1", 0) - pin_voltage(&topo, frame, "C1", 1);
        assert_close(vc, expected, 5e-3, &format!("v_C({t})"));
    }
}

#[test]
fn rl_current_rise() {
    // τ = L/R = 1 ms; i(t) = 0.1·(1 − e^(−t/τ))
    let doc = series_loop(
        (K::DcVoltage, &[("voltage", "1")]),
        &[
            (K::Resistor, &[("resistance", "10")]),
            (K::Inductor, &[("inductance", "10m")]),
        ],
    );
    let frames = transient(&doc, 2e-3, 1e-5).unwrap();
    let topo = Simulator::new(&doc).unwrap().topology();
    let frame = frames
        .iter()
        .min_by(|a, b| (a.time - 1e-3).abs().total_cmp(&(b.time - 1e-3).abs()))
        .unwrap();
    let expected = 0.1 * (1.0 - (-1.0f64).exp());
    assert_close(current(&topo, frame, "L1"), expected, 5e-3, "i_L(1ms)");
}

#[test]
fn diode_forward_bias() {
    let doc = series_loop(
        (K::DcVoltage, &[("voltage", "5")]),
        &[(K::Resistor, &[("resistance", "1k")]), (K::Diode, &[])],
    );
    let mut sim = Simulator::new(&doc).unwrap();
    let frame = sim.dc_operating_point().unwrap();
    let topo = sim.topology();
    let vd = pin_voltage(&topo, &frame, "D1", 0) - pin_voltage(&topo, &frame, "D1", 1);
    assert!((0.55..0.80).contains(&vd), "diyot gerilimi {vd}");
    // KCL: dirençten geçen akım = diyottan geçen akım
    assert_close(
        current(&topo, &frame, "D1"),
        current(&topo, &frame, "R1"),
        1e-6,
        "seri akım",
    );
}

#[test]
fn red_led_with_resistor() {
    let doc = series_loop(
        (K::DcVoltage, &[("voltage", "5")]),
        &[
            (K::Resistor, &[("resistance", "330")]),
            (K::Led, &[("color", "red")]),
        ],
    );
    let mut sim = Simulator::new(&doc).unwrap();
    let frame = sim.dc_operating_point().unwrap();
    let topo = sim.topology();
    let vf = pin_voltage(&topo, &frame, "LED1", 0) - pin_voltage(&topo, &frame, "LED1", 1);
    assert!((1.70..1.85).contains(&vf), "LED gerilimi {vf}");
    assert_close(
        current(&topo, &frame, "LED1"),
        (5.0 - vf) / 330.0,
        1e-6,
        "I(LED)",
    );
}

#[test]
fn reverse_diode_blocks() {
    let doc = series_loop(
        (K::DcVoltage, &[("voltage", "-5")]),
        &[(K::Resistor, &[("resistance", "1k")]), (K::Diode, &[])],
    );
    let mut sim = Simulator::new(&doc).unwrap();
    let frame = sim.dc_operating_point().unwrap();
    let topo = sim.topology();
    assert!(current(&topo, &frame, "D1").abs() < 1e-8);
}

#[test]
fn current_source_into_resistor() {
    // Ok pin 0'a (üst) doğru: akım üst hatta çıkar, dirençten geçip geri döner.
    let doc = series_loop(
        (K::DcCurrent, &[("current", "2m")]),
        &[(K::Resistor, &[("resistance", "1k")])],
    );
    let mut sim = Simulator::new(&doc).unwrap();
    let frame = sim.dc_operating_point().unwrap();
    let topo = sim.topology();
    assert_close(pin_voltage(&topo, &frame, "R1", 0), 2.0, 1e-6, "V(R1)");
    assert_close(current(&topo, &frame, "R1"), 2e-3, 1e-6, "I(R1)");
}

#[test]
fn ammeter_and_voltmeter() {
    let mut b = DocBuilder::new();
    let v = b.add(K::DcVoltage, Point(0, 2), 0, &[("voltage", "9")]);
    let am = b.add(K::Ammeter, Point(6, 0), 0, &[]);
    let r = b.add(K::Resistor, Point(12, 0), 0, &[("resistance", "3k")]);
    let vm = b.add(K::Voltmeter, Point(12, -4), 0, &[]);
    let g = b.add(K::Ground, Point(0, 8), 0, &[]);
    b.wire(b.pin(v, 0), b.pin(am, 0));
    b.wire(b.pin(am, 1), b.pin(r, 0));
    b.wire(b.pin(r, 1), Point(16, 6));
    b.wire(Point(16, 6), Point(0, 6));
    b.wire(b.pin(v, 1), Point(0, 6));
    b.wire(Point(0, 6), b.pin(g, 0));
    b.wire(b.pin(vm, 0), b.pin(r, 0));
    b.wire(b.pin(vm, 1), b.pin(r, 1));

    let mut sim = Simulator::new(&b.finish()).unwrap();
    let frame = sim.dc_operating_point().unwrap();
    let topo = sim.topology();
    assert_close(current(&topo, &frame, "AM1"), 3e-3, 1e-6, "ampermetre");
    let reading = pin_voltage(&topo, &frame, "VM1", 0) - pin_voltage(&topo, &frame, "VM1", 1);
    assert_close(reading, 9.0, 1e-6, "voltmetre");
    assert_eq!(current(&topo, &frame, "VM1"), 0.0);
}

#[test]
fn switch_controls_current() {
    let doc = series_loop(
        (K::DcVoltage, &[("voltage", "5")]),
        &[(K::Switch, &[]), (K::Resistor, &[("resistance", "1k")])],
    );
    let mut sim = Simulator::new(&doc).unwrap();
    sim.step(1e-3).unwrap();
    let topo = sim.topology();
    assert!(
        current(&topo, &sim.frame(), "R1").abs() < 1e-9,
        "açık anahtar"
    );

    assert!(sim.set_toggle("S1", true));
    sim.step(1e-3).unwrap();
    assert_close(
        current(&topo, &sim.frame(), "R1"),
        5e-3,
        1e-4,
        "kapalı anahtar",
    );
}

#[test]
fn capacitor_keeps_charge_after_switch_opens() {
    let doc = series_loop(
        (K::DcVoltage, &[("voltage", "5")]),
        &[
            (K::Switch, &[]),
            (K::Resistor, &[("resistance", "1k")]),
            (K::Capacitor, &[("capacitance", "1µ")]),
        ],
    );
    let mut sim = Simulator::new(&doc).unwrap();
    let topo = sim.topology();
    sim.set_toggle("S1", true);
    for _ in 0..1000 {
        sim.step(1e-5).unwrap(); // 10 ms = 10τ
    }
    sim.set_toggle("S1", false);
    for _ in 0..100 {
        sim.step(1e-5).unwrap();
    }
    let f = sim.frame();
    let vc = pin_voltage(&topo, &f, "C1", 0) - pin_voltage(&topo, &f, "C1", 1);
    assert_close(vc, 5.0, 1e-3, "yük korunmalı");
}

#[test]
fn ac_source_follows_sine() {
    let doc = series_loop(
        (
            K::AcVoltage,
            &[
                ("amplitude", "2"),
                ("frequency", "1k"),
                ("offset", "0"),
                ("phase", "0"),
            ],
        ),
        &[(K::Resistor, &[("resistance", "1k")])],
    );
    let frames = transient(&doc, 1e-3, 5e-5).unwrap();
    let topo = Simulator::new(&doc).unwrap().topology();
    let peak = frames
        .iter()
        .find(|f| (f.time - 2.5e-4).abs() < 1e-9)
        .expect("t = 0.25 ms karesi");
    assert_close(pin_voltage(&topo, peak, "R1", 0), 2.0, 1e-6, "tepe");
}

#[test]
fn parallel_voltage_sources_are_singular() {
    let mut b = DocBuilder::new();
    let v1 = b.add(K::DcVoltage, Point(0, 2), 0, &[("voltage", "5")]);
    let v2 = b.add(K::DcVoltage, Point(6, 2), 0, &[("voltage", "3")]);
    let g = b.add(K::Ground, Point(0, 8), 0, &[]);
    b.wire(b.pin(v1, 0), b.pin(v2, 0));
    b.wire(b.pin(v1, 1), b.pin(v2, 1));
    b.wire(b.pin(v1, 1), b.pin(g, 0));
    let mut sim = Simulator::new(&b.finish()).unwrap();
    assert_eq!(
        sim.dc_operating_point().unwrap_err(),
        CircuitError::Singular
    );
}

#[test]
fn invalid_value_is_reported() {
    let doc = series_loop(
        (K::DcVoltage, &[("voltage", "5")]),
        &[(K::Resistor, &[("resistance", "bir kilo")])],
    );
    assert!(matches!(
        Simulator::new(&doc),
        Err(CircuitError::InvalidValue { component, .. }) if component == "R1"
    ));
}
