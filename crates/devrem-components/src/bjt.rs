//! Ebers-Moll (taşıma modeli) bipolar transistör.

use crate::diode::THERMAL_VOLTAGE;

/// BJT modeli. Pin sırası: kollektör, baz, emetör.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BjtModel {
    /// NPN için +1, PNP için −1.
    pub polarity: f64,
    /// Doyma akımı (A).
    pub saturation_current: f64,
    /// İleri yön akım kazancı (β).
    pub beta_f: f64,
    /// Ters yön akım kazancı.
    pub beta_r: f64,
}

/// Çalışma noktasında terminal akımları ve türevleri.
///
/// Akımlar terminale giren yöndedir. Türevler kutupluluktan bağımsız olarak
/// gerçek düğüm gerilimlerine göredir (`vbe = vB − vE`, `vbc = vB − vC`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BjtEval {
    pub ic: f64,
    pub ib: f64,
    pub dic_dvbe: f64,
    pub dic_dvbc: f64,
    pub dib_dvbe: f64,
    pub dib_dvbc: f64,
}

impl BjtModel {
    pub fn new(polarity: f64, beta: f64) -> Self {
        Self {
            polarity,
            saturation_current: 1e-14,
            beta_f: beta,
            beta_r: 1.0,
        }
    }

    fn junction(&self, v: f64) -> (f64, f64) {
        let e = (v / THERMAL_VOLTAGE).exp();
        (
            self.saturation_current * (e - 1.0),
            self.saturation_current * e / THERMAL_VOLTAGE,
        )
    }

    /// `vbe`, `vbc` gerçek (kutupluluk uygulanmamış) gerilimlerdir.
    pub fn evaluate(&self, vbe: f64, vbc: f64) -> BjtEval {
        let p = self.polarity;
        let (i_f, g_f) = self.junction(p * vbe);
        let (i_r, g_r) = self.junction(p * vbc);
        let ic = i_f - i_r - i_r / self.beta_r;
        let ib = i_f / self.beta_f + i_r / self.beta_r;
        BjtEval {
            ic: p * ic,
            ib: p * ib,
            dic_dvbe: g_f,
            dic_dvbc: -g_r * (1.0 + 1.0 / self.beta_r),
            dib_dvbe: g_f / self.beta_f,
            dib_dvbc: g_r / self.beta_r,
        }
    }

    /// Jonksiyon gerilimi adım sınırlaması (gerçek gerilimler üzerinde).
    pub fn limit(&self, v_new: f64, v_old: f64) -> f64 {
        let p = self.polarity;
        let vt = THERMAL_VOLTAGE;
        let v_crit = vt * (vt / (std::f64::consts::SQRT_2 * self.saturation_current)).ln();
        let (n, o) = (p * v_new, p * v_old);
        let limited = if n > v_crit && (n - o).abs() > 2.0 * vt {
            if o > 0.0 {
                let arg = 1.0 + (n - o) / vt;
                if arg > 0.0 { o + vt * arg.ln() } else { v_crit }
            } else {
                vt * (n / vt).ln()
            }
        } else {
            n
        };
        p * limited
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn forward_active_gain() {
        let m = BjtModel::new(1.0, 100.0);
        let e = m.evaluate(0.65, -5.0);
        assert!((e.ic / e.ib - 100.0).abs() < 1e-6);
    }

    #[test]
    fn pnp_mirrors_npn() {
        let n = BjtModel::new(1.0, 100.0).evaluate(0.65, -5.0);
        let p = BjtModel::new(-1.0, 100.0).evaluate(-0.65, 5.0);
        assert!((n.ic + p.ic).abs() < 1e-18);
        assert!((n.dic_dvbe - p.dic_dvbe).abs() < 1e-12);
    }
}
