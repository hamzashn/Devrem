//! Bileşen türleri ve pin konumları.
//!
//! Pin konumları `packages/library/components.json` ile aynı olmalıdır;
//! arayüz o dosyayı kullanır. Bir test iki tanımın eşleştiğini doğrular.

use crate::geometry::Point;
use serde::{Deserialize, Serialize};

/// Bileşen türü.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ComponentKind {
    Ground,
    Resistor,
    Capacitor,
    Inductor,
    DcVoltage,
    DcCurrent,
    AcVoltage,
    Diode,
    Led,
    Switch,
    Voltmeter,
    Ammeter,
    Npn,
    Pnp,
    Nmos,
    Pmos,
    OpAmp,
    And,
    Or,
    Not,
    Nand,
    Nor,
    Xor,
    Xnor,
    DFlipFlop,
    JkFlipFlop,
    TFlipFlop,
    LogicInput,
    LogicProbe,
    Clock,
    Potentiometer,
    Zener,
    PushButton,
    SpdtSwitch,
    Lamp,
    PulseVoltage,
    Timer555,
    SevenSegment,
    HexDecoder,
}

const HORIZONTAL: &[Point] = &[Point(-2, 0), Point(2, 0)];
const VERTICAL: &[Point] = &[Point(0, -2), Point(0, 2)];
const SINGLE: &[Point] = &[Point(0, 0)];
/// Üst uç, sol uç (kontrol), alt uç: NPN (C, B, E), NMOS (D, G, S).
const TRANSISTOR_TOP_FIRST: &[Point] = &[Point(1, -2), Point(-2, 0), Point(1, 2)];
/// PNP (C, B, E) ve PMOS (D, G, S): emetör / kaynak üstte.
const TRANSISTOR_BOTTOM_FIRST: &[Point] = &[Point(1, 2), Point(-2, 0), Point(1, -2)];
/// Op-amp (−, +, çıkış) ve iki girişli kapılar (A, B, Y).
const TWO_IN_ONE_OUT: &[Point] = &[Point(-3, -1), Point(-3, 1), Point(3, 0)];
const ONE_IN_ONE_OUT: &[Point] = &[Point(-3, 0), Point(3, 0)];
/// D / T flip-flop: (D|T, CLK, Q, Q̄).
const FLIP_FLOP: &[Point] = &[Point(-3, -1), Point(-3, 1), Point(3, -1), Point(3, 1)];
/// JK flip-flop: (J, CLK, K, Q, Q̄).
const JK_FLIP_FLOP: &[Point] = &[
    Point(-3, -1),
    Point(-3, 0),
    Point(-3, 1),
    Point(3, -1),
    Point(3, 1),
];
const OUTPUT_PIN: &[Point] = &[Point(2, 0)];
/// Potansiyometre: (A, B, orta uç).
const POTENTIOMETER: &[Point] = &[Point(-2, 0), Point(2, 0), Point(0, -2)];
/// İki yollu anahtar: (ortak, A, B).
const SPDT: &[Point] = &[Point(-2, 0), Point(2, -1), Point(2, 1)];
/// 555 zamanlayıcı, veri sayfası sırasıyla:
/// 1 GND, 2 TRIG, 3 OUT, 4 RESET, 5 CTRL, 6 THR, 7 DIS, 8 VCC.
const TIMER_555: &[Point] = &[
    Point(1, 4),
    Point(-3, 1),
    Point(3, -1),
    Point(-1, -4),
    Point(-1, 4),
    Point(-3, -1),
    Point(3, 1),
    Point(1, -4),
];
/// 7 segment gösterge: a, b, c, d, e, f, g.
const SEVEN_SEGMENT: &[Point] = &[
    Point(-3, -3),
    Point(-3, -2),
    Point(-3, -1),
    Point(-3, 0),
    Point(-3, 1),
    Point(-3, 2),
    Point(-3, 3),
];
/// Onaltılık çözücü: girişler A, B, C, D; çıkışlar a … g.
const HEX_DECODER: &[Point] = &[
    Point(-3, -3),
    Point(-3, -1),
    Point(-3, 1),
    Point(-3, 3),
    Point(3, -3),
    Point(3, -2),
    Point(3, -1),
    Point(3, 0),
    Point(3, 1),
    Point(3, 2),
    Point(3, 3),
];
const INPUT_PIN: &[Point] = &[Point(-2, 0)];

impl ComponentKind {
    pub const ALL: [Self; 39] = [
        Self::Ground,
        Self::Resistor,
        Self::Capacitor,
        Self::Inductor,
        Self::DcVoltage,
        Self::DcCurrent,
        Self::AcVoltage,
        Self::Diode,
        Self::Led,
        Self::Switch,
        Self::Voltmeter,
        Self::Ammeter,
        Self::Npn,
        Self::Pnp,
        Self::Nmos,
        Self::Pmos,
        Self::OpAmp,
        Self::And,
        Self::Or,
        Self::Not,
        Self::Nand,
        Self::Nor,
        Self::Xor,
        Self::Xnor,
        Self::DFlipFlop,
        Self::JkFlipFlop,
        Self::TFlipFlop,
        Self::LogicInput,
        Self::LogicProbe,
        Self::Clock,
        Self::Potentiometer,
        Self::Zener,
        Self::PushButton,
        Self::SpdtSwitch,
        Self::Lamp,
        Self::PulseVoltage,
        Self::Timer555,
        Self::SevenSegment,
        Self::HexDecoder,
    ];

    /// Döndürülmemiş haldeki pin konumları (bileşen merkezine göre).
    ///
    /// İki uçlu bileşenlerde akım yönü pin 0 → pin 1 kabul edilir.
    /// Kaynaklarda pin 0 artı (+) uçtur.
    pub const fn pins(self) -> &'static [Point] {
        match self {
            Self::Ground => SINGLE,
            Self::DcVoltage | Self::DcCurrent | Self::AcVoltage | Self::PulseVoltage => VERTICAL,
            Self::Potentiometer => POTENTIOMETER,
            Self::SpdtSwitch => SPDT,
            Self::Timer555 => TIMER_555,
            Self::SevenSegment => SEVEN_SEGMENT,
            Self::HexDecoder => HEX_DECODER,
            Self::Npn | Self::Nmos => TRANSISTOR_TOP_FIRST,
            Self::Pnp | Self::Pmos => TRANSISTOR_BOTTOM_FIRST,
            Self::OpAmp
            | Self::And
            | Self::Or
            | Self::Nand
            | Self::Nor
            | Self::Xor
            | Self::Xnor => TWO_IN_ONE_OUT,
            Self::Not => ONE_IN_ONE_OUT,
            Self::DFlipFlop | Self::TFlipFlop => FLIP_FLOP,
            Self::JkFlipFlop => JK_FLIP_FLOP,
            Self::LogicInput | Self::Clock => OUTPUT_PIN,
            Self::LogicProbe => INPUT_PIN,
            _ => HORIZONTAL,
        }
    }

    /// Akımını görünmeyen bir beslemeden (toprağa göre) alan çıkış pinleri.
    ///
    /// Tel akımları hesaplanırken bu pinler, toprak gibi akım kaynağı/kuyusu
    /// olarak kabul edilir.
    pub const fn supply_pins(self) -> &'static [usize] {
        match self {
            Self::OpAmp
            | Self::And
            | Self::Or
            | Self::Nand
            | Self::Nor
            | Self::Xor
            | Self::Xnor => &[2],
            Self::Not => &[1],
            Self::DFlipFlop | Self::TFlipFlop => &[2, 3],
            Self::JkFlipFlop => &[3, 4],
            Self::LogicInput | Self::Clock | Self::Ground => &[0],
            Self::Timer555 => &[2],
            Self::HexDecoder => &[4, 5, 6, 7, 8, 9, 10],
            _ => &[],
        }
    }

    /// Otomatik isimlendirmede kullanılan önek (R1, C2 ...).
    pub const fn prefix(self) -> &'static str {
        match self {
            Self::Ground => "GND",
            Self::Resistor => "R",
            Self::Capacitor => "C",
            Self::Inductor => "L",
            Self::DcVoltage | Self::AcVoltage | Self::PulseVoltage => "V",
            Self::DcCurrent => "I",
            Self::Diode | Self::Zener => "D",
            Self::Led => "LED",
            Self::Switch | Self::PushButton | Self::SpdtSwitch => "S",
            Self::Voltmeter => "VM",
            Self::Ammeter => "AM",
            Self::Npn | Self::Pnp => "Q",
            Self::Nmos | Self::Pmos => "M",
            Self::OpAmp => "OP",
            Self::And
            | Self::Or
            | Self::Not
            | Self::Nand
            | Self::Nor
            | Self::Xor
            | Self::Xnor
            | Self::Timer555
            | Self::HexDecoder => "U",
            Self::DFlipFlop | Self::JkFlipFlop | Self::TFlipFlop => "FF",
            Self::LogicInput => "IN",
            Self::LogicProbe => "OUT",
            Self::Clock => "CLK",
            Self::Potentiometer => "RV",
            Self::Lamp => "LP",
            Self::SevenSegment => "DS",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    const LIBRARY_JSON: &str = include_str!("../../../packages/library/components.json");

    #[test]
    fn matches_shared_library_json() {
        let json: serde_json::Map<String, Value> = serde_json::from_str(LIBRARY_JSON).unwrap();
        assert_eq!(
            json.len(),
            ComponentKind::ALL.len(),
            "bileşen sayısı farklı"
        );

        for kind in ComponentKind::ALL {
            let key = serde_json::to_value(kind).unwrap();
            let key = key.as_str().unwrap();
            let def = json.get(key).unwrap_or_else(|| panic!("JSON'da {key} yok"));

            assert_eq!(def["prefix"], kind.prefix(), "{key} öneki");
            let pins: Vec<Point> = serde_json::from_value(def["pins"].clone()).unwrap();
            assert_eq!(pins, kind.pins(), "{key} pinleri");
        }
    }
}
