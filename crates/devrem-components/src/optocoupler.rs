//! Optokuplör: kızılötesi LED ve fototransistör.
//!
//! LED akımı, akım aktarım oranı (CTR) kadar kollektör akımına dönüşür.
//! Fototransistör doyuma girdiğinde akım, kollektör–emetör gerilimiyle
//! yumuşakça sınırlanır: `Ic = CTR · I_LED · tanh(Vce / VCE_SAT)`.
//! Giriş ve çıkış birbirinden elektriksel olarak yalıtılmıştır.

use crate::diode::DiodeModel;

/// Fototransistörün doyma bölgesinin genişliği (V).
pub const VCE_SAT: f64 = 0.2;
/// Kızılötesi LED'in 10 mA'deki ileri yön gerilimi (V).
const IR_FORWARD_VOLTAGE: f64 = 1.2;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OptocouplerModel {
    pub led: DiodeModel,
    /// Akım aktarım oranı (1.0 = %100).
    pub ctr: f64,
}

/// Fototransistör akımı ve türevleri.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PhotoEval {
    /// Kollektörden giren akım.
    pub ic: f64,
    /// ∂Ic/∂Vd (LED gerilimi)
    pub d_vd: f64,
    /// ∂Ic/∂Vce
    pub d_vce: f64,
}

impl OptocouplerModel {
    pub fn new(ctr: f64) -> Self {
        Self {
            led: DiodeModel::with_forward_voltage(IR_FORWARD_VOLTAGE),
            ctr,
        }
    }

    /// LED gerilimi `vd` ve kollektör–emetör gerilimi `vce` için
    /// fototransistör akımı.
    pub fn photo(&self, vd: f64, vce: f64) -> PhotoEval {
        let (id, gd) = self.led.evaluate(vd);
        let t = (vce / VCE_SAT).tanh();
        PhotoEval {
            ic: self.ctr * id * t,
            d_vd: self.ctr * gd * t,
            d_vce: self.ctr * id * (1.0 - t * t) / VCE_SAT,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transfers_current_with_ctr() {
        let m = OptocouplerModel::new(0.5);
        let (id, _) = m.led.evaluate(IR_FORWARD_VOLTAGE);
        assert!((id - 0.01).abs() < 1e-6, "LED 1.2 V'ta 10 mA: {id}");
        // Doyma dışında (Vce ≫ VCE_SAT) Ic = CTR · I_LED
        let e = m.photo(IR_FORWARD_VOLTAGE, 5.0);
        assert!((e.ic - 0.005).abs() < 1e-6, "{}", e.ic);
        // Vce = 0 iken akım akmaz
        assert_eq!(m.photo(IR_FORWARD_VOLTAGE, 0.0).ic, 0.0);
    }
}
