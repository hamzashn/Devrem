//! Dijital bileşenler.
//!
//! Lojik bileşenler analog devreyle birlikte çalışır: girişler gerilim eşiğine
//! göre okunur, çıkışlar küçük iç dirençli gerilim kaynağıdır. Durum her zaman
//! adımının başında bir önceki çözüme göre güncellenir.

/// Lojik besleme gerilimi (V).
pub const LOGIC_HIGH: f64 = 5.0;
/// Giriş eşiği (V).
pub const LOGIC_THRESHOLD: f64 = 2.5;
/// Çıkış iç direnci (Ω).
pub const LOGIC_OUTPUT_RESISTANCE: f64 = 20.0;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LogicKind {
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
    Input,
    Clock {
        frequency: f64,
    },
    Probe,
    /// 7 segment gösterge (yalnızca giriş).
    Display,
    /// 4 bit girişi (A = LSB) onaltılık 7 segment koduna çevirir.
    HexDecoder,
}

/// Onaltılık rakamların 7 segment kodları (bit 0 = a … bit 6 = g).
const HEX_SEGMENTS: [u8; 16] = [
    0b011_1111, 0b000_0110, 0b101_1011, 0b100_1111, 0b110_0110, 0b110_1101, 0b111_1101, 0b000_0111,
    0b111_1111, 0b110_1111, 0b111_0111, 0b111_1100, 0b011_1001, 0b101_1110, 0b111_1001, 0b111_0001,
];

#[derive(Debug, Clone, PartialEq)]
pub struct LogicDevice {
    pub kind: LogicKind,
    /// Giriş pinlerinin indeksleri.
    pub inputs: &'static [usize],
    /// Çıkış pinlerinin indeksleri.
    pub outputs: &'static [usize],
    /// Çıkış seviyeleri (`outputs` sırasıyla).
    pub levels: Vec<bool>,
    /// Flip-flop durumu (Q).
    q: bool,
    /// Önceki saat seviyesi (kenar algılama).
    previous_clock: bool,
    /// `false` iken saat kenarları yok sayılır (başlangıç durumu oturtulurken).
    edges_enabled: bool,
    /// Lojik giriş bileşeninin konumu.
    pub high: bool,
}

impl LogicDevice {
    pub fn new(kind: LogicKind, high: bool) -> Self {
        let (inputs, outputs): (&[usize], &[usize]) = match kind {
            LogicKind::Not => (&[0], &[1]),
            LogicKind::DFlipFlop | LogicKind::TFlipFlop => (&[0, 1], &[2, 3]),
            LogicKind::JkFlipFlop => (&[0, 1, 2], &[3, 4]),
            LogicKind::Input | LogicKind::Clock { .. } => (&[], &[0]),
            LogicKind::Probe => (&[0], &[]),
            LogicKind::Display => (&[0, 1, 2, 3, 4, 5, 6], &[]),
            LogicKind::HexDecoder => (&[0, 1, 2, 3], &[4, 5, 6, 7, 8, 9, 10]),
            _ => (&[0, 1], &[2]),
        };
        let mut device = Self {
            kind,
            inputs,
            outputs,
            levels: vec![false; outputs.len()],
            q: false,
            previous_clock: false,
            edges_enabled: true,
            high,
        };
        device.prime(&[false; 3], 0.0);
        device
    }

    /// Saat kenarı üretmeden giriş seviyelerini öğrenir ve çıkışları hesaplar.
    /// Simülasyon başlarken flip-flop'ların sahte kenar görmemesi için kullanılır.
    pub fn prime(&mut self, inputs: &[bool], time: f64) -> bool {
        self.edges_enabled = false;
        let changed = self.update(inputs, time);
        self.edges_enabled = true;
        changed
    }

    /// Girişlere (`inputs` sırasıyla) ve zamana göre çıkışları günceller.
    /// Herhangi bir çıkış değiştiyse `true` döner.
    pub fn update(&mut self, inputs: &[bool], time: f64) -> bool {
        let at = |i: usize| inputs.get(i).copied().unwrap_or(false);
        let (a, b) = (at(0), at(1));
        let next: Vec<bool> = match self.kind {
            LogicKind::And => vec![a && b],
            LogicKind::Or => vec![a || b],
            LogicKind::Not => vec![!a],
            LogicKind::Nand => vec![!(a && b)],
            LogicKind::Nor => vec![!(a || b)],
            LogicKind::Xor => vec![a ^ b],
            LogicKind::Xnor => vec![!(a ^ b)],
            LogicKind::DFlipFlop => {
                if self.rising(b) {
                    self.q = a;
                }
                vec![self.q, !self.q]
            }
            LogicKind::TFlipFlop => {
                if self.rising(b) && a {
                    self.q = !self.q;
                }
                vec![self.q, !self.q]
            }
            LogicKind::JkFlipFlop => {
                let (j, clk, k) = (a, b, at(2));
                if self.rising(clk) {
                    self.q = match (j, k) {
                        (true, true) => !self.q,
                        (true, false) => true,
                        (false, true) => false,
                        (false, false) => self.q,
                    };
                }
                vec![self.q, !self.q]
            }
            LogicKind::Input => vec![self.high],
            LogicKind::Clock { frequency } => vec![(time * frequency).fract() < 0.5],
            LogicKind::Probe | LogicKind::Display => vec![],
            LogicKind::HexDecoder => {
                let value = (0..4).fold(0usize, |acc, i| acc | (usize::from(at(i)) << i));
                let code = HEX_SEGMENTS[value];
                (0..7).map(|bit| code & (1 << bit) != 0).collect()
            }
        };
        let changed = next != self.levels;
        self.levels = next;
        changed
    }

    fn rising(&mut self, clock: bool) -> bool {
        let rising = self.edges_enabled && clock && !self.previous_clock;
        self.previous_clock = clock;
        rising
    }

    /// Saat üreteci için önerilen en büyük zaman adımı.
    pub fn max_step(&self) -> Option<f64> {
        match self.kind {
            LogicKind::Clock { frequency } => Some(1.0 / (40.0 * frequency)),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gate_truth_tables() {
        let cases = [
            (LogicKind::And, [false, false, false, true]),
            (LogicKind::Or, [false, true, true, true]),
            (LogicKind::Nand, [true, true, true, false]),
            (LogicKind::Nor, [true, false, false, false]),
            (LogicKind::Xor, [false, true, true, false]),
            (LogicKind::Xnor, [true, false, false, true]),
        ];
        for (kind, expected) in cases {
            let mut g = LogicDevice::new(kind, false);
            for (i, (a, b)) in [(false, false), (false, true), (true, false), (true, true)]
                .into_iter()
                .enumerate()
            {
                g.update(&[a, b], 0.0);
                assert_eq!(g.levels[0], expected[i], "{kind:?} {a} {b}");
            }
        }
    }

    #[test]
    fn hex_decoder_segments() {
        let mut d = LogicDevice::new(LogicKind::HexDecoder, false);
        // 1 → yalnızca b ve c
        d.update(&[true, false, false, false], 0.0);
        assert_eq!(
            d.levels,
            vec![false, true, true, false, false, false, false]
        );
        // 8 → tüm segmentler
        d.update(&[false, false, false, true], 0.0);
        assert!(d.levels.iter().all(|&l| l));
    }

    #[test]
    fn d_flip_flop_samples_on_rising_edge() {
        let mut ff = LogicDevice::new(LogicKind::DFlipFlop, false);
        ff.update(&[true, false], 0.0);
        assert!(!ff.levels[0], "kenar yokken değişmemeli");
        ff.update(&[true, true], 0.0);
        assert_eq!(ff.levels, vec![true, false]);
        ff.update(&[false, true], 0.0);
        assert!(ff.levels[0], "saat yüksekken değişmemeli");
    }

    #[test]
    fn t_flip_flop_divides_clock() {
        let mut ff = LogicDevice::new(LogicKind::TFlipFlop, false);
        let mut toggles = 0;
        assert!(
            !ff.prime(&[true, true], 0.0),
            "hazırlıkta kenar sayılmamalı"
        );
        for i in 0..8 {
            let clk = i % 2 == 1;
            if ff.update(&[true, clk], 0.0) {
                toggles += 1;
            }
        }
        assert_eq!(toggles, 4);
    }
}
