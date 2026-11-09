//! SPICE netlist (`.cir`) dışa aktarımı.
//!
//! Analog bileşenler standart SPICE elemanlarına çevrilir. SPICE'ta doğrudan
//! karşılığı olmayan bileşenler (lojik kapılar, flip-flop'lar, 555, ölçüm
//! aletleri) açıklama satırı olarak yazılır.

use devrem_components::{Device, SWITCH_R_OFF, SWITCH_R_ON, Waveform};
use devrem_core::{CircuitError, ComponentKind, Document, Netlist};
use std::collections::BTreeMap;
use std::fmt::Write;

/// Sayıyı SPICE'ın okuyabileceği biçimde yazar. Kayan nokta artıklarını
/// (ör. 10 µ → 9.999999999999999e-6) önlemek için 12 anlamlı basamağa yuvarlar.
fn num(v: f64) -> String {
    let v: f64 = format!("{v:.11e}").parse().unwrap_or(v);
    if v == 0.0 {
        "0".into()
    } else if v.abs() < 1e-3 || v.abs() >= 1e6 {
        format!("{v:e}")
    } else {
        format!("{v}")
    }
}

fn node_name(node: usize) -> String {
    if node == 0 {
        "0".into()
    } else {
        format!("N{node:03}")
    }
}

/// SPICE eleman adı: tür harfiyle başlamalıdır (ör. LED1 → DLED1).
fn element_name(letter: char, id: &str) -> String {
    if id.to_ascii_uppercase().starts_with(letter) {
        id.to_owned()
    } else {
        format!("{letter}{id}")
    }
}

/// Belgeyi SPICE netlist metnine çevirir.
pub fn to_spice(doc: &Document) -> Result<String, CircuitError> {
    let netlist = Netlist::build(doc)?;
    let mut out = String::new();
    let mut models: BTreeMap<String, String> = BTreeMap::new();
    let mut ac_source = false;
    let mut max_frequency: f64 = 0.0;

    let _ = writeln!(out, "* {}", doc.meta.name);
    let _ = writeln!(
        out,
        "* Devrem {} ile dışa aktarıldı",
        env!("CARGO_PKG_VERSION")
    );
    let _ = writeln!(out);

    for element in &netlist.elements {
        let component = &doc.components[element.component];
        let id = &component.id;
        let n: Vec<String> = element.nodes.iter().map(|&n| node_name(n)).collect();
        let device = Device::from_component(component)?.expect("toprak netlistte yok");
        let line = match &device {
            Device::Resistor { resistance } => {
                format!(
                    "{} {} {} {}",
                    element_name('R', id),
                    n[0],
                    n[1],
                    num(*resistance)
                )
            }
            Device::Capacitor { capacitance } => {
                format!(
                    "{} {} {} {}",
                    element_name('C', id),
                    n[0],
                    n[1],
                    num(*capacitance)
                )
            }
            Device::Inductor { inductance } => {
                format!(
                    "{} {} {} {}",
                    element_name('L', id),
                    n[0],
                    n[1],
                    num(*inductance)
                )
            }
            Device::VoltageSource { waveform } => {
                let spec = match *waveform {
                    Waveform::Dc(v) => format!("DC {}", num(v)),
                    Waveform::Sine {
                        amplitude,
                        frequency,
                        offset,
                        phase,
                    } => {
                        ac_source = true;
                        max_frequency = max_frequency.max(frequency);
                        let deg = phase.to_degrees();
                        format!(
                            "SIN({} {} {} 0 0 {}) AC {} {}",
                            num(offset),
                            num(amplitude),
                            num(frequency),
                            num(deg),
                            num(amplitude),
                            num(deg)
                        )
                    }
                    Waveform::Pulse {
                        low,
                        high,
                        frequency,
                        duty,
                    } => {
                        max_frequency = max_frequency.max(frequency);
                        let period = 1.0 / frequency;
                        format!(
                            "PULSE({} {} 0 1n 1n {} {})",
                            num(low),
                            num(high),
                            num(duty * period),
                            num(period)
                        )
                    }
                };
                format!("{} {} {} {}", element_name('V', id), n[0], n[1], spec)
            }
            // Devrem: akım pin 1'den kaynağın içinden pin 0'a akar.
            // SPICE: I n+ n- değer, akım n+'dan kaynağın içinden n-'ye akar.
            Device::CurrentSource { current } => {
                format!(
                    "{} {} {} DC {}",
                    element_name('I', id),
                    n[1],
                    n[0],
                    num(*current)
                )
            }
            Device::Diode { model } => {
                let (name, spec) = match (component.kind, model.breakdown) {
                    (_, Some(bv)) => (
                        format!("DZ_{}", num(bv).replace('.', "V")),
                        format!("D(IS=1e-12 N=1 BV={} IBV=5m)", num(bv)),
                    ),
                    (ComponentKind::Led, None) => {
                        let color = component
                            .props
                            .get("color")
                            .and_then(|v| v.as_str())
                            .unwrap_or("red")
                            .to_ascii_uppercase();
                        (
                            format!("DLED_{color}"),
                            format!(
                                "D(IS={} N={})",
                                num(model.saturation_current),
                                num(model.emission)
                            ),
                        )
                    }
                    _ => ("D1N4148".into(), "D(IS=2.52n N=1.752)".into()),
                };
                models.insert(name.clone(), spec);
                format!("{} {} {} {}", element_name('D', id), n[0], n[1], name)
            }
            Device::Switch { closed } => {
                let r = if *closed { SWITCH_R_ON } else { SWITCH_R_OFF };
                format!(
                    "{} {} {} {} ; anahtar ({})",
                    element_name('R', id),
                    n[0],
                    n[1],
                    num(r),
                    if *closed { "kapalı" } else { "açık" }
                )
            }
            Device::Spdt { to_b } => {
                let (ra, rb) = if *to_b {
                    (SWITCH_R_OFF, SWITCH_R_ON)
                } else {
                    (SWITCH_R_ON, SWITCH_R_OFF)
                };
                let name = element_name('R', id);
                format!(
                    "{name}A {} {} {}\n{name}B {} {} {}",
                    n[0],
                    n[1],
                    num(ra),
                    n[0],
                    n[2],
                    num(rb)
                )
            }
            Device::Potentiometer {
                resistance,
                position,
            } => {
                let name = element_name('R', id);
                let r_aw = (resistance * position).max(1e-3);
                let r_wb = (resistance * (1.0 - position)).max(1e-3);
                format!(
                    "{name}A {} {} {}\n{name}B {} {} {}",
                    n[0],
                    n[2],
                    num(r_aw),
                    n[2],
                    n[1],
                    num(r_wb)
                )
            }
            Device::Ammeter => format!(
                "{} {} {} DC 0 ; ampermetre",
                element_name('V', id),
                n[0],
                n[1]
            ),
            Device::Voltmeter => format!("* {id}: voltmetre, V({}) − V({})", n[0], n[1]),
            Device::Bjt { model } => {
                let kind = if model.polarity > 0.0 { "NPN" } else { "PNP" };
                let name = format!("Q{kind}_B{}", num(model.beta_f));
                models.insert(
                    name.clone(),
                    format!(
                        "{kind}(IS={} BF={} BR={})",
                        num(model.saturation_current),
                        num(model.beta_f),
                        num(model.beta_r)
                    ),
                );
                format!(
                    "{} {} {} {} {}",
                    element_name('Q', id),
                    n[0],
                    n[1],
                    n[2],
                    name
                )
            }
            Device::Mosfet { model } => {
                let (kind, vto) = if model.polarity > 0.0 {
                    ("NMOS", model.threshold)
                } else {
                    ("PMOS", -model.threshold)
                };
                let name = format!("M{kind}_{}", element_name('M', id));
                models.insert(
                    name.clone(),
                    format!(
                        "{kind}(LEVEL=1 VTO={} KP={} LAMBDA={})",
                        num(vto),
                        num(model.k),
                        num(model.lambda)
                    ),
                );
                // Gövde kaynağa bağlı
                format!(
                    "{} {} {} {} {} {}",
                    element_name('M', id),
                    n[0],
                    n[1],
                    n[2],
                    n[2],
                    name
                )
            }
            Device::OpAmp { gain, vsat } => format!(
                "{} {} 0 {} {} {} ; op-amp (çıkış sınırı ±{} V SPICE'ta modellenmedi)",
                element_name('E', id),
                n[2],
                n[1],
                n[0],
                num(*gain),
                num(*vsat)
            ),
            Device::Logic(_) | Device::Timer(_) => {
                let kind = serde_json::to_value(component.kind)
                    .ok()
                    .and_then(|v| v.as_str().map(str::to_owned))
                    .unwrap_or_default();
                format!(
                    "* {id} ({kind}): SPICE dışa aktarımında desteklenmiyor; düğümler: {}",
                    n.join(" ")
                )
            }
        };
        let _ = writeln!(out, "{line}");
    }

    if !models.is_empty() {
        let _ = writeln!(out);
        for (name, spec) in &models {
            let _ = writeln!(out, ".model {name} {spec}");
        }
    }

    let _ = writeln!(out);
    let _ = writeln!(out, ".op");
    let stop = if max_frequency > 0.0 {
        5.0 / max_frequency
    } else {
        3.0 * doc.simulation.time_scale
    };
    let _ = writeln!(out, ".tran {} {}", num(stop / 1000.0), num(stop));
    if ac_source {
        let ac = &doc.simulation.ac;
        let _ = writeln!(
            out,
            ".ac dec {} {} {}",
            ac.points_per_decade,
            num(ac.start),
            num(ac.stop)
        );
    }
    let _ = writeln!(out, ".end");
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use devrem_core::{DocBuilder, Point};

    #[test]
    fn exports_divider() {
        let mut b = DocBuilder::new();
        let v = b.add(
            ComponentKind::DcVoltage,
            Point(0, 2),
            0,
            &[("voltage", "12")],
        );
        let r1 = b.add(
            ComponentKind::Resistor,
            Point(6, 0),
            0,
            &[("resistance", "2k")],
        );
        let led = b.add(ComponentKind::Led, Point(12, 0), 0, &[("color", "green")]);
        let g = b.add(ComponentKind::Ground, Point(0, 8), 0, &[]);
        b.wire(b.pin(v, 0), b.pin(r1, 0));
        b.wire(b.pin(r1, 1), b.pin(led, 0));
        b.wire(b.pin(led, 1), Point(14, 6));
        b.wire(Point(14, 6), b.pin(g, 0));
        b.wire(b.pin(v, 1), b.pin(g, 0));
        let text = to_spice(&b.finish()).unwrap();
        assert!(text.contains("V1 N001 0 DC 12"), "{text}");
        assert!(text.contains("R1 N001 N002 2000"), "{text}");
        assert!(text.contains("DLED1 N002 0 DLED_GREEN"), "{text}");
        assert!(text.contains(".model DLED_GREEN D(IS="), "{text}");
        assert!(text.trim_end().ends_with(".end"));
    }

    #[test]
    fn numbers() {
        assert_eq!(num(0.0), "0");
        assert_eq!(num(4700.0), "4700");
        assert_eq!(num(1e-7), "1e-7");
        assert_eq!(num(2.2e6), "2.2e6");
        assert_eq!(num(10.0 * 1e-6), "1e-5");
        assert_eq!(num(0.1 + 0.2), "0.3");
    }
}
