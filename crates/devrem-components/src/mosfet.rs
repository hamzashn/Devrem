//! Shichman-Hodges (SPICE seviye 1) MOSFET modeli. Gövde kaynağa bağlıdır.

/// MOSFET modeli. Pin sırası: savak (drain), kapı (gate), kaynak (source).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MosfetModel {
    /// NMOS için +1, PMOS için −1.
    pub polarity: f64,
    /// Eşik gerilimi büyüklüğü (V).
    pub threshold: f64,
    /// İletim katsayısı K = µCox·W/L (A/V²).
    pub k: f64,
    /// Kanal boyu modülasyonu (1/V).
    pub lambda: f64,
}

/// Savak akımı ve türevleri (gerçek düğüm gerilimlerine göre).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MosfetEval {
    /// Savaktan giren akım.
    pub id: f64,
    /// ∂Id/∂Vgs
    pub gm: f64,
    /// ∂Id/∂Vds
    pub gds: f64,
}

impl MosfetModel {
    pub fn new(polarity: f64, threshold: f64, k: f64) -> Self {
        Self {
            polarity,
            threshold,
            k,
            lambda: 0.01,
        }
    }

    /// Normalize (NMOS) gerilimlerle, `vds ≥ 0` için.
    fn forward(&self, vgs: f64, vds: f64) -> (f64, f64, f64) {
        let vov = vgs - self.threshold;
        if vov <= 0.0 {
            return (0.0, 0.0, 0.0);
        }
        let clm = 1.0 + self.lambda * vds;
        if vds < vov {
            let base = vov * vds - 0.5 * vds * vds;
            (
                self.k * base * clm,
                self.k * vds * clm,
                self.k * (vov - vds) * clm + self.k * base * self.lambda,
            )
        } else {
            let base = 0.5 * vov * vov;
            (
                self.k * base * clm,
                self.k * vov * clm,
                self.k * base * self.lambda,
            )
        }
    }

    /// `vgs = vG − vS`, `vds = vD − vS` (gerçek gerilimler).
    pub fn evaluate(&self, vgs: f64, vds: f64) -> MosfetEval {
        let p = self.polarity;
        let (vgs, vds) = (p * vgs, p * vds);
        let (id, gm, gds) = if vds >= 0.0 {
            self.forward(vgs, vds)
        } else {
            // Savak ve kaynak yer değiştirir: id(vgs, vds) = −f(vgs − vds, −vds)
            let (i, m, d) = self.forward(vgs - vds, -vds);
            (-i, -m, m + d)
        };
        MosfetEval {
            id: p * id,
            gm,
            gds,
        }
    }
}

/// Newton-Raphson adımını sınırlar.
pub fn limit_step(v_new: f64, v_old: f64, max_step: f64) -> f64 {
    v_new.clamp(v_old - max_step, v_old + max_step)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn saturation_current() {
        let m = MosfetModel::new(1.0, 2.0, 0.1);
        let e = m.evaluate(4.0, 10.0);
        // 0.1/2 · 2² · (1 + 0.1) = 0.22 A
        assert!((e.id - 0.22).abs() < 1e-12);
        assert_eq!(m.evaluate(1.0, 10.0).id, 0.0);
    }

    #[test]
    fn pmos_mirrors_nmos() {
        let n = MosfetModel::new(1.0, 2.0, 0.1).evaluate(4.0, 1.0);
        let p = MosfetModel::new(-1.0, 2.0, 0.1).evaluate(-4.0, -1.0);
        assert!((n.id + p.id).abs() < 1e-15);
        assert!((n.gm - p.gm).abs() < 1e-15);
    }

    #[test]
    fn reverse_is_antisymmetric() {
        // Vgs = Vgd olduğunda (simetrik çalışma) akım ters işaretli olmalı.
        let m = MosfetModel::new(1.0, 1.0, 0.1);
        let fwd = m.evaluate(3.0, 0.5).id;
        let rev = m.evaluate(2.5, -0.5).id;
        assert!((fwd + rev).abs() < 1e-12, "{fwd} {rev}");
    }
}
