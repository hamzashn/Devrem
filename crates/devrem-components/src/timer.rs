//! 555 zamanlayıcının davranışsal modeli.
//!
//! İç yapı: CTRL ucunda VCC'nin 2/3'ünü veren direnç bölücü, iki karşılaştırıcı
//! (TRIG < CTRL/2 → kur, THR > CTRL → sıfırla), SR mandal, çıkış katı ve
//! deşarj transistörü. Karşılaştırıcılar lojik bileşenler gibi her zaman
//! adımının başında önceki çözüme göre değerlendirilir.

/// Pin indeksleri (veri sayfası numarası − 1).
pub mod pins {
    pub const GND: usize = 0;
    pub const TRIG: usize = 1;
    pub const OUT: usize = 2;
    pub const RESET: usize = 3;
    pub const CTRL: usize = 4;
    pub const THR: usize = 5;
    pub const DIS: usize = 6;
    pub const VCC: usize = 7;
}

/// İç bölücünün her bir direnci (Ω).
pub const DIVIDER_RESISTANCE: f64 = 5e3;
/// Çıkış katının iç direnci (Ω).
pub const OUTPUT_RESISTANCE: f64 = 10.0;
/// İletimdeki deşarj transistörünün direnci (Ω).
pub const DISCHARGE_RESISTANCE: f64 = 10.0;
/// RESET ucunun VCC'ye iç çekme direnci (Ω). Gerçek 555'te yoktur; RESET
/// bağlanmadan bırakıldığında entegrenin çalışmasını sağlar.
pub const RESET_PULLUP: f64 = 100e3;
/// RESET eşiği (V).
const RESET_THRESHOLD: f64 = 0.7;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Timer555 {
    /// Mandal çıkışı: `true` iken OUT yüksek, deşarj kapalı.
    pub q: bool,
    /// Çıkışın yüksek seviyesi (VCC − GND), V.
    pub level: f64,
}

impl Timer555 {
    /// `v(pin)` ile verilen uç gerilimlerine göre mandalı günceller.
    /// Mandal durumu değiştiyse `true` döner.
    pub fn update(&mut self, v: impl Fn(usize) -> f64) -> bool {
        let gnd = v(pins::GND);
        let rel = |pin| v(pin) - gnd;
        let ctrl = rel(pins::CTRL);
        let previous = self.q;
        if rel(pins::RESET) < RESET_THRESHOLD {
            self.q = false;
        } else if rel(pins::TRIG) < ctrl / 2.0 {
            self.q = true;
        } else if rel(pins::THR) > ctrl {
            self.q = false;
        }
        self.level = rel(pins::VCC).max(0.0);
        previous != self.q
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn latch_behaviour() {
        let mut t = Timer555::default();
        // VCC 9 V, CTRL 6 V, RESET yüksek
        let volts = |trig: f64, thr: f64| {
            move |pin: usize| match pin {
                pins::VCC | pins::RESET => 9.0,
                pins::CTRL => 6.0,
                pins::TRIG => trig,
                pins::THR => thr,
                _ => 0.0,
            }
        };
        assert!(t.update(volts(2.0, 2.0)), "TRIG < 3 V → kur");
        assert!(t.q);
        assert!(!t.update(volts(4.0, 4.0)), "arada → durum korunur");
        assert!(t.update(volts(7.0, 7.0)), "THR > 6 V → sıfırla");
        assert!(!t.q);
        assert_eq!(t.level, 9.0);
    }
}
