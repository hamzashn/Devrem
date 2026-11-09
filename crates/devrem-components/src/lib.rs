//! Devrem bileşen modelleri.
//!
//! Şemadaki bileşen özelliklerini (ör. `"resistance": "4k7"`) ayrıştırıp
//! simülasyonda kullanılacak tipli modellere çevirir.

pub mod bjt;
pub mod diode;
pub mod logic;
pub mod mosfet;
pub mod timer;
pub mod waveform;

pub use bjt::{BjtEval, BjtModel};
pub use diode::DiodeModel;
pub use logic::{LogicDevice, LogicKind};
pub use mosfet::{MosfetEval, MosfetModel};
pub use timer::Timer555;
pub use waveform::Waveform;

use devrem_core::{CircuitError, Component, ComponentKind, parse_value};
use serde_json::Value;

/// Kapalı anahtarın direnci (Ω).
pub const SWITCH_R_ON: f64 = 1e-3;
/// Açık anahtarın direnci (Ω).
pub const SWITCH_R_OFF: f64 = 1e12;

/// Simülasyon modeli.
#[derive(Debug, Clone, PartialEq)]
pub enum Device {
    Resistor {
        resistance: f64,
    },
    Capacitor {
        capacitance: f64,
    },
    Inductor {
        inductance: f64,
    },
    /// Pin 0 artı uçtur.
    VoltageSource {
        waveform: Waveform,
    },
    /// Akımı kaynağın içinden pin 1'den pin 0'a (ok yönünde) iter.
    CurrentSource {
        current: f64,
    },
    Diode {
        model: DiodeModel,
    },
    Switch {
        closed: bool,
    },
    /// İdeal voltmetre: devreye bağlanmaz, yalnızca ölçer.
    Voltmeter,
    /// İdeal ampermetre: 0 V'luk kaynak gibi davranır.
    Ammeter,
    /// Pin sırası: kollektör, baz, emetör.
    Bjt {
        model: BjtModel,
    },
    /// Pin sırası: savak, kapı, kaynak.
    Mosfet {
        model: MosfetModel,
    },
    /// Pin sırası: eviren giriş (−), evirmeyen giriş (+), çıkış.
    /// Çıkış toprağa göre `±vsat` ile yumuşakça sınırlanır.
    OpAmp {
        gain: f64,
        vsat: f64,
    },
    Logic(LogicDevice),
    /// Pin sırası: A, B, orta uç. `position` (0–1) orta ucun A'dan uzaklığı.
    Potentiometer {
        resistance: f64,
        position: f64,
    },
    /// Pin sırası: ortak, A, B. `to_b` iken ortak uç B'ye bağlıdır.
    Spdt {
        to_b: bool,
    },
    /// Pin sırası için bkz. [`timer::pins`].
    Timer(Timer555),
}

/// Potansiyometre uçlarının en küçük direnci (Ω); sıfır dirence düşmesin.
pub const POT_MIN_RESISTANCE: f64 = 1e-3;

/// Op-amp çıkışı ve eğimi: `vsat · tanh(A·vd / vsat)`.
pub fn opamp_output(gain: f64, vsat: f64, vd: f64) -> (f64, f64) {
    let t = (gain * vd / vsat).tanh();
    // Doymada eğim sıfıra inmesin; Newton-Raphson'ın girişlerle bağı korunur.
    let slope = (gain * (1.0 - t * t)).max(gain * 1e-9);
    (vsat * t, slope)
}

impl Device {
    /// Bir şema bileşeninden model oluşturur. Toprak için `None` döner.
    pub fn from_component(c: &Component) -> Result<Option<Self>, CircuitError> {
        let p = Props(c);
        let device = match c.kind {
            ComponentKind::Ground => return Ok(None),
            ComponentKind::Resistor => Self::Resistor {
                resistance: p.positive("resistance", "1k")?,
            },
            ComponentKind::Capacitor => Self::Capacitor {
                capacitance: p.positive("capacitance", "100µ")?,
            },
            ComponentKind::Inductor => Self::Inductor {
                inductance: p.positive("inductance", "100m")?,
            },
            ComponentKind::DcVoltage => Self::VoltageSource {
                waveform: Waveform::Dc(p.number("voltage", "5")?),
            },
            ComponentKind::DcCurrent => Self::CurrentSource {
                current: p.number("current", "10m")?,
            },
            ComponentKind::AcVoltage => Self::VoltageSource {
                waveform: Waveform::Sine {
                    amplitude: p.number("amplitude", "5")?,
                    frequency: p.positive("frequency", "1")?,
                    offset: p.number("offset", "0")?,
                    phase: p.number("phase", "0")?.to_radians(),
                },
            },
            ComponentKind::Diode => Self::Diode {
                model: DiodeModel::SILICON,
            },
            ComponentKind::Led => {
                let color = p.text("color", "red");
                let model = DiodeModel::led(&color).ok_or_else(|| p.invalid("color", &color))?;
                Self::Diode { model }
            }
            ComponentKind::Switch => Self::Switch {
                closed: p.boolean("closed")?,
            },
            ComponentKind::Voltmeter => Self::Voltmeter,
            ComponentKind::Ammeter => Self::Ammeter,
            ComponentKind::Potentiometer => Self::Potentiometer {
                resistance: p.positive("resistance", "10k")?,
                position: (p.number("position", "50")? / 100.0).clamp(0.0, 1.0),
            },
            ComponentKind::Zener => Self::Diode {
                model: DiodeModel::zener(p.positive("breakdown", "5.1")?),
            },
            ComponentKind::PushButton => Self::Switch { closed: false },
            ComponentKind::SpdtSwitch => Self::Spdt {
                to_b: p.boolean("toB")?,
            },
            ComponentKind::Lamp => {
                let voltage = p.positive("voltage", "12")?;
                let power = p.positive("power", "1")?;
                Self::Resistor {
                    resistance: voltage * voltage / power,
                }
            }
            ComponentKind::PulseVoltage => {
                let duty = p.number("duty", "50")? / 100.0;
                if !(0.0..=1.0).contains(&duty) {
                    return Err(CircuitError::OutOfRange {
                        component: c.id.clone(),
                        prop: "duty".into(),
                    });
                }
                Self::VoltageSource {
                    waveform: Waveform::Pulse {
                        low: p.number("low", "0")?,
                        high: p.number("high", "5")?,
                        frequency: p.positive("frequency", "1")?,
                        duty,
                    },
                }
            }
            ComponentKind::Timer555 => Self::Timer(Timer555::default()),
            ComponentKind::SevenSegment => Self::Logic(LogicDevice::new(LogicKind::Display, false)),
            ComponentKind::HexDecoder => {
                Self::Logic(LogicDevice::new(LogicKind::HexDecoder, false))
            }
            ComponentKind::Npn | ComponentKind::Pnp => {
                let polarity = if c.kind == ComponentKind::Npn {
                    1.0
                } else {
                    -1.0
                };
                Self::Bjt {
                    model: BjtModel::new(polarity, p.positive("beta", "100")?),
                }
            }
            ComponentKind::Nmos | ComponentKind::Pmos => {
                let polarity = if c.kind == ComponentKind::Nmos {
                    1.0
                } else {
                    -1.0
                };
                Self::Mosfet {
                    model: MosfetModel::new(
                        polarity,
                        p.number("vth", "2")?.abs(),
                        p.positive("k", "100m")?,
                    ),
                }
            }
            ComponentKind::OpAmp => Self::OpAmp {
                gain: p.positive("gain", "100k")?,
                vsat: p.positive("vsat", "15")?,
            },
            ComponentKind::Clock => Self::Logic(LogicDevice::new(
                LogicKind::Clock {
                    frequency: p.positive("frequency", "1")?,
                },
                false,
            )),
            ComponentKind::LogicInput => {
                Self::Logic(LogicDevice::new(LogicKind::Input, p.boolean("high")?))
            }
            kind => {
                let logic = match kind {
                    ComponentKind::And => LogicKind::And,
                    ComponentKind::Or => LogicKind::Or,
                    ComponentKind::Not => LogicKind::Not,
                    ComponentKind::Nand => LogicKind::Nand,
                    ComponentKind::Nor => LogicKind::Nor,
                    ComponentKind::Xor => LogicKind::Xor,
                    ComponentKind::Xnor => LogicKind::Xnor,
                    ComponentKind::DFlipFlop => LogicKind::DFlipFlop,
                    ComponentKind::JkFlipFlop => LogicKind::JkFlipFlop,
                    ComponentKind::TFlipFlop => LogicKind::TFlipFlop,
                    _ => LogicKind::Probe,
                };
                Self::Logic(LogicDevice::new(logic, false))
            }
        };
        Ok(Some(device))
    }

    /// Doğrusal olmayan model mi (Newton-Raphson gerektirir)?
    pub fn is_nonlinear(&self) -> bool {
        matches!(
            self,
            Self::Diode { .. }
                | Self::Bjt { .. }
                | Self::Mosfet { .. }
                | Self::OpAmp { .. }
                | Self::Timer(_)
        )
    }
}

/// Bileşen özelliklerini okuma yardımcısı.
struct Props<'a>(&'a Component);

impl Props<'_> {
    fn invalid(&self, prop: &str, value: &str) -> CircuitError {
        CircuitError::InvalidValue {
            component: self.0.id.clone(),
            prop: prop.to_owned(),
            value: value.to_owned(),
        }
    }

    fn number(&self, key: &str, default: &str) -> Result<f64, CircuitError> {
        let value = match self.0.props.get(key) {
            None => parse_value(default).map_err(|_| self.invalid(key, default))?,
            Some(Value::Number(n)) => n.as_f64().ok_or_else(|| self.invalid(key, "?"))?,
            Some(Value::String(s)) => parse_value(s).map_err(|_| self.invalid(key, s))?,
            Some(other) => return Err(self.invalid(key, &other.to_string())),
        };
        if value.is_finite() {
            Ok(value)
        } else {
            Err(self.invalid(key, &value.to_string()))
        }
    }

    fn positive(&self, key: &str, default: &str) -> Result<f64, CircuitError> {
        let value = self.number(key, default)?;
        if value > 0.0 {
            Ok(value)
        } else {
            Err(CircuitError::OutOfRange {
                component: self.0.id.clone(),
                prop: key.to_owned(),
            })
        }
    }

    fn boolean(&self, key: &str) -> Result<bool, CircuitError> {
        match self.0.props.get(key) {
            None => Ok(false),
            Some(Value::Bool(b)) => Ok(*b),
            Some(other) => Err(self.invalid(key, &other.to_string())),
        }
    }

    fn text(&self, key: &str, default: &str) -> String {
        match self.0.props.get(key) {
            Some(Value::String(s)) => s.clone(),
            _ => default.to_owned(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use devrem_core::Point;
    use std::collections::BTreeMap;

    const LIBRARY_JSON: &str = include_str!("../../../packages/library/components.json");

    fn component(kind: ComponentKind, props: BTreeMap<String, Value>) -> Component {
        Component {
            id: "X1".into(),
            kind,
            pos: Point(0, 0),
            rot: 0,
            props,
        }
    }

    #[test]
    fn library_defaults_are_valid() {
        let json: serde_json::Map<String, Value> = serde_json::from_str(LIBRARY_JSON).unwrap();
        for kind in ComponentKind::ALL {
            let key = serde_json::to_value(kind).unwrap();
            let def = &json[key.as_str().unwrap()];
            let props: BTreeMap<String, Value> =
                serde_json::from_value(def["props"].clone()).unwrap();
            let result = Device::from_component(&component(kind, props));
            assert!(result.is_ok(), "{kind:?}: {result:?}");
        }
    }

    #[test]
    fn parses_values_with_prefixes() {
        let c = component(
            ComponentKind::Resistor,
            [("resistance".into(), "4k7".into())].into(),
        );
        assert_eq!(
            Device::from_component(&c).unwrap(),
            Some(Device::Resistor { resistance: 4700.0 })
        );
    }

    #[test]
    fn rejects_bad_values() {
        let bad = component(
            ComponentKind::Resistor,
            [("resistance".into(), "abc".into())].into(),
        );
        assert!(matches!(
            Device::from_component(&bad),
            Err(CircuitError::InvalidValue { .. })
        ));

        let zero = component(
            ComponentKind::Capacitor,
            [("capacitance".into(), "0".into())].into(),
        );
        assert!(matches!(
            Device::from_component(&zero),
            Err(CircuitError::OutOfRange { .. })
        ));
    }
}
